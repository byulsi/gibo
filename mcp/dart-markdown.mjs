// Convert DART document XML to Markdown with korean-dart-mcp's own converter.
// The converter drops <LIBRARY> as if it were an image block, but DART reports wrap
// whole chapters in it ("II. 사업의 내용", financial statement notes). Unwrapping the
// tag first keeps those chapters.
// Usage: node dart-markdown.mjs <path to dart-xml.js> < {"docs": [{"name", "xml"}]}
import { pathToFileURL } from "node:url";

const { dartXmlToMarkdown } = await import(pathToFileURL(process.argv[2]).href);
let input = "";
for await (const chunk of process.stdin) input += chunk;
const { docs } = JSON.parse(input);
const parts = docs.map(({ name, xml }) => {
  const markdown = dartXmlToMarkdown(xml.replace(/<\/?LIBRARY\b[^>]*>/g, ""));
  return docs.length > 1 ? `<!-- ${name} -->\n${markdown}` : markdown;
});
process.stdout.write(parts.join("\n\n"));
