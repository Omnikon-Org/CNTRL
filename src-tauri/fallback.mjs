import { chromium } from 'playwright';

async function main() {
  const url = process.argv[2];
  if (!url) {
    console.error("No URL provided");
    process.exit(1);
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  try {
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 30000 });
    
    // Inject <base> tag to fix relative links for CSS/assets inside the iframe
    await page.evaluate((url) => {
      let base = document.querySelector('base');
      if (!base) {
        base = document.createElement('base');
        document.head.prepend(base);
      }
      base.href = url;
    }, url);

    const html = await page.content();
    console.log(html);
  } catch (error) {
    console.error(`Error loading page: ${error}`);
  } finally {
    await browser.close();
  }
}

main();
