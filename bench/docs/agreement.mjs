/// Helpers for long agreements: a cover page, a table of contents, and
/// articles of numbered sections.
import { longDate } from '../lib/format.mjs';

/// "ARTICLE V" + "REPRESENTATIONS AND WARRANTIES OF SELLER", then sections
/// "5.1 Organization. ..." with any sub-paragraphs indented below.
export function writeArticles(flow, articles, { size, startArticle = 1 } = {}) {
  const roman = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII', 'XIII', 'XIV'];
  articles.forEach(([title, sections], articleIndex) => {
    const number = startArticle + articleIndex;
    flow.heading(`ARTICLE ${roman[number - 1]}`, { level: 3, align: 'center', after: 0, before: 12 });
    flow.paragraph(title.toUpperCase(), { face: 'sans-bold', size: 10, align: 'center', after: 8, keepWithNext: 40 });
    sections.forEach(([heading, body, items = []], sectionIndex) => {
      flow.paragraph([{ text: `${number}.${sectionIndex + 1} ${heading}. `, face: 'sans-bold' }, { text: body }], { size });
      items.forEach((item, itemIndex) => flow.paragraph(`(${String.fromCharCode(97 + itemIndex)}) ${item}`, { indent: 22, size }));
    });
  });
}

/// A table of contents page listing articles and schedules.
export function tableOfContents(flow, articles, schedules, { startArticle = 1 } = {}) {
  const roman = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII', 'XIII', 'XIV'];
  flow.heading('TABLE OF CONTENTS', { level: 2, align: 'center' });
  articles.forEach(([title, sections], articleIndex) => {
    const number = startArticle + articleIndex;
    flow.paragraph(`ARTICLE ${roman[number - 1]} - ${title}`, { face: 'sans-bold', size: 9, after: 1 });
    flow.paragraph(sections.map(([heading], sectionIndex) => `${number}.${sectionIndex + 1} ${heading}`).join('; '), { size: 8.5, indent: 14, after: 4 });
  });
  flow.paragraph('SCHEDULES', { face: 'sans-bold', size: 9, after: 1, before: 4 });
  flow.paragraph(schedules.join('; '), { size: 8.5, indent: 14 });
}

export function coverPage(flow, { title, between, dated, footer }) {
  const page = flow.page;
  let y = 220;
  page.textCenter(306, y, title, { face: 'sans-bold', size: 20 });
  y += 40;
  for (const line of between) {
    page.textCenter(306, y, line.text, { face: line.bold ? 'sans-bold' : 'serif', size: line.bold ? 12 : 11 });
    y += line.bold ? 20 : 18;
  }
  y += 24;
  page.textCenter(306, y, dated ?? '', { face: 'serif', size: 12 });
  if (footer) page.textCenter(306, 700, footer, { face: 'serif', size: 9, grey: 0.3 });
  flow.pageBreak();
}

export { longDate };
