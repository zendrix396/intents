const fs = require("fs");
const path = require("path");

// Function to extract title from MDX content
function extractTitle(content) {
  const titleMatch = content.match(/^#\s+(.+)$/m);
  return titleMatch ? titleMatch[1] : "Untitled";
}

// Function to generate navigation from docs directory
function generateDocsNav() {
  const docsDir = path.join(__dirname, "../src/app/docs");
  const nav = [];

  try {
    const items = fs.readdirSync(docsDir, { withFileTypes: true });

    for (const item of items) {
      if (item.isDirectory() && item.name !== "[[...slug]]") {
        const pagePath = path.join(docsDir, item.name, "page.mdx");
        if (fs.existsSync(pagePath)) {
          const content = fs.readFileSync(pagePath, "utf8");
          const title = extractTitle(content);
          nav.push({
            slug: item.name,
            title: title,
            path: `/docs/${item.name}`,
          });
        }
      }
    }
  } catch (error) {
    console.error("Error reading docs directory:", error);
  }

  return nav;
}

// Generate and save navigation
const nav = generateDocsNav();
const navPath = path.join(__dirname, "../src/lib/docs-nav.json");
fs.writeFileSync(navPath, JSON.stringify(nav, null, 2));
console.log("Generated docs navigation:", nav);
