"""Gibo additions to the KRX server: technical_indicators and usage notes.

Indicators are computed here from KRX daily prices, so the price series never
passes through the conversation. Conventions: Wilder smoothing for RSI and ATR
(seeded with a simple average), population standard deviation for Bollinger
bands, sample standard deviation of log returns for realized volatility, and
252 trading days a year. Checked against OpenBB technical on the same prices.
"""
import datetime
import json
import math

INSTRUCTIONS = """[Gibo] 사용 안내
- 국내 종목·지수의 시세, 거래량, 투자자별 수급, 공매도, PER/PBR은 이 KRX 도구를 기본으로 쓴다. OpenBB(야후) 국내 시세는 날짜별 종가·거래량이 KRX와 다를 때가 있다.
- 이동평균, RSI, MACD, ATR, 볼린저 밴드, 변동성, 기간 수익률, 52주 고저, 최대 낙폭은 technical_indicators로 계산한다. 시세를 받아 직접 계산하거나 다른 도구에 다시 보내지 않는다.
- get_market_net_purchases_of_equities처럼 시장 전체 종목을 돌려주는 도구는 응답이 매우 크다. 필요한 시장·기간만 요청한다."""

TOOLS = [{
    "name": "technical_indicators",
    "description": (
        "국내 종목이나 지수의 KRX 일별 시세로 기술적 지표를 계산해 최신값만 돌려줍니다. "
        "시세 원본은 대화에 싣지 않습니다. 이동평균(5/20/60/120/200일)과 이격도, RSI(14), "
        "MACD(12,26,9), ATR(14), 볼린저 밴드(20,2), 실현 변동성(20/60일, 연율), "
        "기간 수익률(1주~1년), 52주 고가·저가, 기간 최대 낙폭, 거래량 배율, 최근 5거래일."),
    "inputSchema": {
        "type": "object",
        "properties": {
            "ticker": {"type": "string", "description": "종목코드 6자리(예: 005930) 또는 지수코드(예: 1001 코스피)"},
            "kind": {"type": "string", "enum": ["stock", "index"], "default": "stock",
                     "description": "stock=종목, index=지수"},
            "end_date": {"type": "string", "description": "기준일 YYYYMMDD (생략 시 오늘)"},
            "start_date": {"type": "string",
                           "description": "시작일 YYYYMMDD (생략 시 기준일 400일 전, 200일 이동평균과 52주 계산에 충분)"},
        },
        "required": ["ticker"],
    },
}]


def rows_from_krx(text):
    """KRX stock prices come as a list of {날짜, ...}; index prices as {date: {...}}."""
    data = json.loads(text).get("data")
    if isinstance(data, dict):
        rows = [{"날짜": day[:10], **values} for day, values in data.items()]
    else:
        rows = list(data or [])
    rows = [r for r in rows if r.get("종가") not in (None, 0)]
    rows.sort(key=lambda r: r["날짜"])
    return rows


def sma(values, n):
    return sum(values[-n:]) / n if len(values) >= n else None


def ema_series(values, n):
    if len(values) < n:
        return []
    alpha = 2 / (n + 1)
    current = sum(values[:n]) / n
    out = [current]
    for value in values[n:]:
        current = alpha * value + (1 - alpha) * current
        out.append(current)
    return out


def wilder(values, n):
    """Wilder's smoothing seeded with the simple average of the first n values."""
    if len(values) < n:
        return None
    current = sum(values[:n]) / n
    for value in values[n:]:
        current = (current * (n - 1) + value) / n
    return current


def rsi(closes, n=14):
    changes = [b - a for a, b in zip(closes, closes[1:])]
    gain = wilder([max(c, 0) for c in changes], n)
    loss = wilder([max(-c, 0) for c in changes], n)
    if gain is None:
        return None
    return 100.0 if loss == 0 else 100 - 100 / (1 + gain / loss)


def atr(highs, lows, closes, n=14):
    ranges = [max(h - l, abs(h - pc), abs(l - pc)) for h, l, pc in zip(highs[1:], lows[1:], closes)]
    return wilder(ranges, n)


def stdev(values):
    if len(values) < 2:
        return None
    mean = sum(values) / len(values)
    return math.sqrt(sum((v - mean) ** 2 for v in values) / (len(values) - 1))


def realized_volatility(closes, n):
    if len(closes) < n + 1:
        return None
    logs = [math.log(b / a) for a, b in zip(closes[-n - 1:], closes[-n:])]
    return stdev(logs) * math.sqrt(252)


def pct(a, b):
    return None if a is None or b in (None, 0) else (a / b - 1) * 100


def rounded(value, digits=2):
    return None if value is None else round(value, digits)


def compute(rows):
    closes = [float(r["종가"]) for r in rows]
    highs = [float(r.get("고가") or r["종가"]) for r in rows]
    lows = [float(r.get("저가") or r["종가"]) for r in rows]
    volumes = [float(r.get("거래량") or 0) for r in rows]
    last = closes[-1]
    result = {"기준일": rows[-1]["날짜"], "시작일": rows[0]["날짜"], "거래일수": len(rows), "종가": last}

    result["이동평균"] = {f"{n}일": rounded(sma(closes, n)) for n in (5, 20, 60, 120, 200)}
    result["이격도_%"] = {f"{n}일": rounded(pct(last, sma(closes, n))) for n in (20, 60, 120, 200)}
    result["RSI_14"] = rounded(rsi(closes))

    fast, slow = ema_series(closes, 12), ema_series(closes, 26)
    if slow:
        macd = [f - s for f, s in zip(fast[len(fast) - len(slow):], slow)]
        signal = ema_series(macd, 9)
        if signal:
            result["MACD"] = {"MACD": rounded(macd[-1]), "시그널": rounded(signal[-1]),
                              "히스토그램": rounded(macd[-1] - signal[-1])}

    average_range = atr(highs, lows, closes)
    result["ATR_14"] = {"값": rounded(average_range), "종가대비_%": rounded(average_range / last * 100 if average_range else None)}

    middle = sma(closes, 20)
    if middle is not None:
        window = closes[-20:]
        width = 2 * math.sqrt(sum((v - middle) ** 2 for v in window) / 20)  # population, as Bollinger
        upper, lower = middle + width, middle - width
        result["볼린저_20_2"] = {"상단": rounded(upper), "중심": rounded(middle), "하단": rounded(lower),
                               "%B": rounded((last - lower) / (upper - lower) if upper != lower else None, 3)}

    volatility = {n: realized_volatility(closes, n) for n in (20, 60)}
    result["실현변동성_연율_%"] = {f"{n}일": rounded(v * 100 if v is not None else None) for n, v in volatility.items()}
    result["수익률_%"] = {label: rounded(pct(last, closes[-n - 1]) if len(closes) > n else None)
                       for label, n in (("1주", 5), ("1개월", 21), ("3개월", 63), ("6개월", 126), ("1년", 252))}

    year = rows[-252:]
    high_row = max(year, key=lambda r: float(r.get("고가") or r["종가"]))
    low_row = min(year, key=lambda r: float(r.get("저가") or r["종가"]))
    year_high = float(high_row.get("고가") or high_row["종가"])
    year_low = float(low_row.get("저가") or low_row["종가"])
    result["52주"] = {"고가": year_high, "고가일": high_row["날짜"], "저가": year_low, "저가일": low_row["날짜"],
                     "고가대비_%": rounded(pct(last, year_high)), "저가대비_%": rounded(pct(last, year_low)),
                     "기간": f"{year[0]['날짜']}~{year[-1]['날짜']} ({len(year)}거래일)"}

    peak, drawdown, peak_day, trough_day, worst = closes[0], 0.0, rows[0]["날짜"], rows[0]["날짜"], (0.0, "", "")
    for row, close in zip(rows, closes):
        if close > peak:
            peak, peak_day = close, row["날짜"]
        drawdown = close / peak - 1
        if drawdown < worst[0]:
            worst = (drawdown, peak_day, row["날짜"])
    result["최대낙폭_종가기준_%"] = {"값": rounded(worst[0] * 100), "고점일": worst[1], "저점일": worst[2]}

    average_volume = sma(volumes, 20)
    result["거래량_20일평균대비_배"] = rounded(volumes[-1] / average_volume if average_volume else None)
    result["최근5거래일"] = [{"날짜": r["날짜"], "종가": r["종가"], "거래량": r.get("거래량")} for r in rows[-5:]]
    result["계산방식"] = ("KRX 일별 시세(종목은 수정주가), RSI·ATR은 Wilder 평활(첫 값은 단순평균), "
                       "볼린저는 모표준편차, 변동성은 로그수익률 표본표준편차×√252, 수익률은 n거래일 전 종가 대비")
    return result


def technical_indicators(server, arguments):
    ticker = str(arguments.get("ticker", "")).strip()
    if not ticker:
        raise RuntimeError("ticker is required")
    end = arguments.get("end_date") or datetime.date.today().strftime("%Y%m%d")
    end_day = datetime.datetime.strptime(end, "%Y%m%d").date()
    start = arguments.get("start_date") or (end_day - datetime.timedelta(days=400)).strftime("%Y%m%d")
    if arguments.get("kind", "stock") == "index":
        text = server.call_tool("get_index_ohlcv", {"ticker": ticker, "start_date": start, "end_date": end,
                                                    "freq": "d"})
    else:
        text = server.call_tool("get_stock_ohlcv", {"ticker": ticker, "start_date": start, "end_date": end,
                                                    "adjusted": True})
    rows = rows_from_krx(text)
    if len(rows) < 2:
        raise RuntimeError(f"{ticker}: {start}~{end} 시세가 없습니다")
    return json.dumps({"ticker": ticker, **compute(rows)}, ensure_ascii=False)


def handle(server, tool, arguments):
    if tool == "technical_indicators":
        return technical_indicators(server, arguments)
    raise RuntimeError(f"unknown tool {tool}")
