/// Minimal, valid Office Open XML writers: DOCX, PPTX, XLSX.
///
/// Each package carries the parts Word, PowerPoint, and Excel themselves
/// require (content types, relationships, styles, and for decks a master,
/// layout, and theme), so the files open in the real applications and not
/// only in the worker's reader. The worker reads DOCX and PPTX through
/// AnyDoc to Markdown - headings, paragraphs, and tables, but not running
/// headers or footers - and XLSX through calamine, one pipe table per
/// sheet, with date-formatted serials rendered as ISO dates. Builders put
/// every fact the gold depends on where those readers look.
import { zip } from './zip.mjs';
import { assertSupported } from './fonts.mjs';
import { toDays } from './format.mjs';

export function xml(text) {
  return String(text).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
}

const XML_HEADER = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>\n';
const REL = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';
const PKG_REL = 'http://schemas.openxmlformats.org/package/2006/relationships';

function relationships(entries) {
  return `${XML_HEADER}<Relationships xmlns="${PKG_REL}">${entries.map(([id, type, target]) => `<Relationship Id="${id}" Type="${REL}/${type}" Target="${target}"/>`).join('')}</Relationships>`;
}

function coreProperties(title, creator) {
  return `${XML_HEADER}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>${xml(title)}</dc:title><dc:creator>${xml(creator)}</dc:creator><dcterms:created xsi:type="dcterms:W3CDTF">2026-01-01T00:00:00Z</dcterms:created><dcterms:modified xsi:type="dcterms:W3CDTF">2026-01-01T00:00:00Z</dcterms:modified></cp:coreProperties>`;
}

// ---------------------------------------------------------------- DOCX

const W = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main';

function wordRuns(content) {
  const runs = typeof content === 'string' ? [{ text: content }] : content;
  return runs.map((run) => {
    assertSupported(run.text);
    const properties = `${run.bold ? '<w:b/>' : ''}${run.italic ? '<w:i/>' : ''}${run.underline ? '<w:u w:val="single"/>' : ''}`;
    return `<w:r>${properties ? `<w:rPr>${properties}</w:rPr>` : ''}<w:t xml:space="preserve">${xml(run.text)}</w:t></w:r>`;
  }).join('');
}

function wordParagraph(content, { style = null, align = null, keepNext = false, pageBreakBefore = false, indent = null } = {}) {
  const properties = [
    style ? `<w:pStyle w:val="${style}"/>` : '',
    keepNext ? '<w:keepNext/>' : '',
    pageBreakBefore ? '<w:pageBreakBefore/>' : '',
    indent ? `<w:ind w:left="${indent}"/>` : '',
    align ? `<w:jc w:val="${align}"/>` : '',
  ].join('');
  return `<w:p>${properties ? `<w:pPr>${properties}</w:pPr>` : ''}${wordRuns(content)}</w:p>`;
}

function wordTable(rows, { header = true, widths = null } = {}) {
  const columns = rows[0].length;
  const grid = widths ?? Array.from({ length: columns }, () => Math.floor(9360 / columns));
  const border = (side) => `<w:${side} w:val="single" w:sz="4" w:space="0" w:color="808080"/>`;
  const body = rows.map((row, rowIndex) => {
    const isHeader = header && rowIndex === 0;
    const cells = row.map((cell, index) => {
      const spec = typeof cell === 'object' && cell !== null ? cell : { text: String(cell) };
      return `<w:tc><w:tcPr><w:tcW w:w="${grid[index]}" w:type="dxa"/>${isHeader ? '<w:shd w:val="clear" w:color="auto" w:fill="D9D9D9"/>' : ''}</w:tcPr>${wordParagraph([{ text: spec.text, bold: isHeader || spec.bold }], { align: spec.align ?? null })}</w:tc>`;
    }).join('');
    return `<w:tr>${isHeader ? '<w:trPr><w:tblHeader/></w:trPr>' : ''}${cells}</w:tr>`;
  }).join('');
  return `<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="0" w:type="auto"/><w:tblBorders>${['top', 'left', 'bottom', 'right', 'insideH', 'insideV'].map(border).join('')}</w:tblBorders><w:tblLook w:val="04A0" w:firstRow="1" w:lastRow="0" w:firstColumn="1" w:lastColumn="0" w:noHBand="0" w:noVBand="1"/></w:tblPr><w:tblGrid>${grid.map((width) => `<w:gridCol w:w="${width}"/>`).join('')}</w:tblGrid>${body}</w:tbl>`;
}

const WORD_STYLES = `${XML_HEADER}<w:styles xmlns:w="${W}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:cs="Times New Roman"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults>`
  + '<w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:qFormat/></w:style>'
  + '<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:jc w:val="center"/><w:spacing w:after="240"/></w:pPr><w:rPr><w:b/><w:caps/><w:sz w:val="32"/></w:rPr></w:style>'
  + '<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr><w:rPr><w:b/><w:sz w:val="26"/></w:rPr></w:style>'
  + '<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/><w:next w:val="Normal"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="80"/><w:outlineLvl w:val="1"/></w:pPr><w:rPr><w:b/><w:sz w:val="23"/></w:rPr></w:style>'
  + '<w:style w:type="paragraph" w:styleId="Header"><w:name w:val="header"/><w:basedOn w:val="Normal"/></w:style>'
  + '<w:style w:type="paragraph" w:styleId="Footer"><w:name w:val="footer"/><w:basedOn w:val="Normal"/></w:style>'
  + '<w:style w:type="table" w:styleId="TableGrid"><w:name w:val="Table Grid"/><w:tblPr><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="auto"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="auto"/></w:tblBorders></w:tblPr></w:style>'
  + '</w:styles>';

/// A Word document. Blocks:
///   { type: 'title' | 'heading' | 'paragraph', text | runs, level, align }
///   { type: 'table', rows, header, widths }
///   { type: 'pageBreak' }
/// `header` / `footer` are plain strings shown on every page.
export function docx({ blocks, header = null, footer = null, title = '', creator = 'InternBench' }) {
  const body = blocks.map((block) => {
    switch (block.type) {
      case 'title':
        return wordParagraph(block.runs ?? block.text, { style: 'Title' });
      case 'heading':
        return wordParagraph(block.runs ?? block.text, { style: `Heading${block.level ?? 1}` });
      case 'paragraph':
        return wordParagraph(block.runs ?? block.text, { align: block.align ?? null, indent: block.indent ?? null });
      case 'table':
        return wordTable(block.rows, block);
      case 'pageBreak':
        return '<w:p><w:r><w:br w:type="page"/></w:r></w:p>';
      default:
        throw new Error(`unknown docx block ${block.type}`);
    }
  }).join('');
  const sectionReferences = `${header ? '<w:headerReference w:type="default" r:id="rIdHeader"/>' : ''}${footer ? '<w:footerReference w:type="default" r:id="rIdFooter"/>' : ''}`;
  const document = `${XML_HEADER}<w:document xmlns:w="${W}" xmlns:r="${REL}"><w:body>${body}<w:sectPr>${sectionReferences}<w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>`;
  const documentRels = [['rIdStyles', 'styles', 'styles.xml']];
  const parts = [];
  if (header) {
    documentRels.push(['rIdHeader', 'header', 'header1.xml']);
    parts.push(['word/header1.xml', `${XML_HEADER}<w:hdr xmlns:w="${W}" xmlns:r="${REL}">${wordParagraph(header, { style: 'Header', align: 'right' })}</w:hdr>`]);
  }
  if (footer) {
    documentRels.push(['rIdFooter', 'footer', 'footer1.xml']);
    parts.push(['word/footer1.xml', `${XML_HEADER}<w:ftr xmlns:w="${W}" xmlns:r="${REL}">${wordParagraph(footer, { style: 'Footer', align: 'center' })}</w:ftr>`]);
  }
  const overrides = [
    ['/word/document.xml', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml'],
    ['/word/styles.xml', 'application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml'],
    ['/docProps/core.xml', 'application/vnd.openxmlformats-package.core-properties+xml'],
    ...(header ? [['/word/header1.xml', 'application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml']] : []),
    ...(footer ? [['/word/footer1.xml', 'application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml']] : []),
  ];
  return zip([
    ['[Content_Types].xml', contentTypes(overrides)],
    ['_rels/.rels', `${XML_HEADER}<Relationships xmlns="${PKG_REL}"><Relationship Id="rId1" Type="${REL}/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>`],
    ['docProps/core.xml', coreProperties(title, creator)],
    ['word/_rels/document.xml.rels', relationships(documentRels)],
    ['word/document.xml', document],
    ['word/styles.xml', WORD_STYLES],
    ...parts,
  ]);
}

function contentTypes(overrides, defaults = []) {
  return `${XML_HEADER}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>${defaults.join('')}${overrides.map(([part, type]) => `<Override PartName="${part}" ContentType="${type}"/>`).join('')}</Types>`;
}

/// The text a reader recovers from DOCX blocks, one block per line and
/// table rows as ` | `-separated cells - the shape of AnyDoc's Markdown
/// without its markup. Headers and footers are not part of it.
export function docxText(blocks) {
  const lines = [];
  for (const block of blocks) {
    if (block.type === 'table') {
      for (const row of block.rows) lines.push(row.map((cell) => (typeof cell === 'object' ? cell.text : String(cell))).join(' | '));
    } else if (block.type !== 'pageBreak') {
      const runs = block.runs ?? [{ text: block.text }];
      lines.push(runs.map((run) => run.text).join(''));
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------- PPTX

const A = 'http://schemas.openxmlformats.org/drawingml/2006/main';
const P = 'http://schemas.openxmlformats.org/presentationml/2006/main';
const SLIDE_NS = `xmlns:a="${A}" xmlns:r="${REL}" xmlns:p="${P}"`;
const EMU = 12700; // per point

function drawingParagraphs(paragraphs, { size = 2000, bold = false } = {}) {
  return paragraphs.map((paragraph) => {
    const spec = typeof paragraph === 'string' ? { text: paragraph } : paragraph;
    assertSupported(spec.text);
    const level = spec.level ? ` lvl="${spec.level}"` : '';
    return `<a:p>${level ? `<a:pPr${level}/>` : ''}<a:r><a:rPr lang="en-US" sz="${spec.size ?? size}"${spec.bold || bold ? ' b="1"' : ''} dirty="0"/><a:t>${xml(spec.text)}</a:t></a:r></a:p>`;
  }).join('');
}

function shape(id, name, { x, y, w, h }, body, { placeholder = null, size, bold } = {}) {
  const nv = placeholder
    ? `<p:nvSpPr><p:cNvPr id="${id}" name="${name}"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="${placeholder}"/></p:nvPr></p:nvSpPr>`
    : `<p:nvSpPr><p:cNvPr id="${id}" name="${name}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>`;
  return `<p:sp>${nv}<p:spPr><a:xfrm><a:off x="${x * EMU}" y="${y * EMU}"/><a:ext cx="${w * EMU}" cy="${h * EMU}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr><p:txBody><a:bodyPr wrap="square"><a:normAutofit/></a:bodyPr><a:lstStyle/>${drawingParagraphs(body, { size, bold })}</p:txBody></p:sp>`;
}

function slideTable(id, { x, y, w, h }, rows) {
  const columns = rows[0].length;
  const columnWidth = Math.floor((w * EMU) / columns);
  const rowHeight = Math.floor((h * EMU) / rows.length);
  const cell = (text, header) => {
    assertSupported(text);
    return `<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="1200"${header ? ' b="1"' : ''} dirty="0"/><a:t>${xml(text)}</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>`;
  };
  return `<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="${id}" name="Table ${id}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="${x * EMU}" y="${y * EMU}"/><a:ext cx="${w * EMU}" cy="${h * EMU}"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table"><a:tbl><a:tblPr firstRow="1" bandRow="1"/><a:tblGrid>${Array.from({ length: columns }, () => `<a:gridCol w="${columnWidth}"/>`).join('')}</a:tblGrid>${rows.map((row, index) => `<a:tr h="${rowHeight}">${row.map((text) => cell(String(text), index === 0)).join('')}</a:tr>`).join('')}</a:tbl></a:graphicData></a:graphic></p:graphicFrame>`;
}

const THEME = `${XML_HEADER}<a:theme xmlns:a="${A}" name="Office Theme"><a:themeElements><a:clrScheme name="Office"><a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1><a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="1F3B57"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="2E6C8E"/></a:accent1><a:accent2><a:srgbClr val="C0612B"/></a:accent2><a:accent3><a:srgbClr val="7A8B3A"/></a:accent3><a:accent4><a:srgbClr val="8E5A9B"/></a:accent4><a:accent5><a:srgbClr val="3E9C9C"/></a:accent5><a:accent6><a:srgbClr val="B8952E"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme><a:fontScheme name="Office"><a:majorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Arial"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln><a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements></a:theme>`;

const MASTER_TITLE = `<p:sp><p:nvSpPr><p:cNvPr id="2" name="Title Placeholder 1"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr><p:spPr><a:xfrm><a:off x="${48 * EMU}" y="${30 * EMU}"/><a:ext cx="${864 * EMU}" cy="${70 * EMU}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody></p:sp>`;

const SLIDE_MASTER = `${XML_HEADER}<p:sldMaster ${SLIDE_NS}><p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>${MASTER_TITLE}</p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz="3200" b="1"/></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz="2000"/></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz="1800"/></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>`;

const SLIDE_LAYOUT = `${XML_HEADER}<p:sldLayout ${SLIDE_NS} type="titleOnly" preserve="1"><p:cSld name="Title Only"><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>`;

/// A deck. Slides: `{ title, body: [string | {text, level, bold}], table:
/// rows, notes }`; 16:9 at 960 x 540 points.
export function pptx({ slides, title = '', creator = 'InternBench' }) {
  const slideXml = slides.map((slide) => {
    let id = 2;
    const shapes = [];
    if (slide.title) shapes.push(shape(id++, 'Title 1', { x: 48, y: 30, w: 864, h: 70 }, [slide.title], { placeholder: 'title', size: slide.titleSize ?? 3200, bold: true }));
    if (slide.subtitle) shapes.push(shape(id++, 'Subtitle 2', { x: 48, y: 110, w: 864, h: 60 }, slide.subtitle, { size: 2000 }));
    if (slide.body?.length) shapes.push(shape(id++, 'Content 3', { x: 48, y: slide.subtitle ? 180 : 115, w: slide.table ? 400 : 864, h: 380 }, slide.body, { size: slide.bodySize ?? 1800 }));
    if (slide.table) shapes.push(slideTable(id++, slide.body?.length ? { x: 470, y: 115, w: 442, h: 360 } : { x: 48, y: 115, w: 864, h: Math.min(390, 28 * slide.table.length) }, slide.table));
    if (slide.footer) shapes.push(shape(id++, 'Footer 9', { x: 48, y: 500, w: 864, h: 24 }, [slide.footer], { size: 1000 }));
    return `${XML_HEADER}<p:sld ${SLIDE_NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>${shapes.join('')}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>`;
  });
  const overrides = [
    ['/ppt/presentation.xml', 'application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml'],
    ['/ppt/slideMasters/slideMaster1.xml', 'application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml'],
    ['/ppt/slideLayouts/slideLayout1.xml', 'application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml'],
    ['/ppt/theme/theme1.xml', 'application/vnd.openxmlformats-officedocument.theme+xml'],
    ['/docProps/core.xml', 'application/vnd.openxmlformats-package.core-properties+xml'],
    ...slides.map((_, index) => [`/ppt/slides/slide${index + 1}.xml`, 'application/vnd.openxmlformats-officedocument.presentationml.slide+xml']),
  ];
  const presentationRels = [
    ['rId1', 'slideMaster', 'slideMasters/slideMaster1.xml'],
    ['rId2', 'theme', 'theme/theme1.xml'],
    ...slides.map((_, index) => [`rId${index + 10}`, 'slide', `slides/slide${index + 1}.xml`]),
  ];
  return zip([
    ['[Content_Types].xml', contentTypes(overrides)],
    ['_rels/.rels', `${XML_HEADER}<Relationships xmlns="${PKG_REL}"><Relationship Id="rId1" Type="${REL}/officeDocument" Target="ppt/presentation.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>`],
    ['docProps/core.xml', coreProperties(title, creator)],
    ['ppt/presentation.xml', `${XML_HEADER}<p:presentation ${SLIDE_NS} saveSubsetFonts="1"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst>${slides.map((_, index) => `<p:sldId id="${256 + index}" r:id="rId${index + 10}"/>`).join('')}</p:sldIdLst><p:sldSz cx="${960 * EMU}" cy="${540 * EMU}"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>`],
    ['ppt/_rels/presentation.xml.rels', relationships(presentationRels)],
    ['ppt/slideMasters/slideMaster1.xml', SLIDE_MASTER],
    ['ppt/slideMasters/_rels/slideMaster1.xml.rels', relationships([['rId1', 'slideLayout', '../slideLayouts/slideLayout1.xml'], ['rId2', 'theme', '../theme/theme1.xml']])],
    ['ppt/slideLayouts/slideLayout1.xml', SLIDE_LAYOUT],
    ['ppt/slideLayouts/_rels/slideLayout1.xml.rels', relationships([['rId1', 'slideMaster', '../slideMasters/slideMaster1.xml']])],
    ['ppt/theme/theme1.xml', THEME],
    ...slideXml.flatMap((content, index) => [
      [`ppt/slides/slide${index + 1}.xml`, content],
      [`ppt/slides/_rels/slide${index + 1}.xml.rels`, relationships([['rId1', 'slideLayout', '../slideLayouts/slideLayout1.xml']])],
    ]),
  ]);
}

/// The text of each slide: title, subtitle, body paragraphs, table rows.
export function pptxText(slides) {
  return slides.map((slide) => [
    slide.title ?? '',
    ...(slide.subtitle ?? []).map((line) => (typeof line === 'string' ? line : line.text)),
    ...(slide.body ?? []).map((line) => (typeof line === 'string' ? line : line.text)),
    ...(slide.table ?? []).map((row) => row.join(' | ')),
    slide.footer ?? '',
  ].filter(Boolean).join('\n'));
}

// ---------------------------------------------------------------- XLSX

const S = 'http://schemas.openxmlformats.org/spreadsheetml/2006/main';

function columnName(index) {
  let name = '';
  let value = index + 1;
  while (value > 0) {
    const remainder = (value - 1) % 26;
    name = String.fromCharCode(65 + remainder) + name;
    value = Math.floor((value - 1) / 26);
  }
  return name;
}

/// Excel's serial day number (1900 date system) for an ISO date.
export function excelSerial(iso) {
  return toDays(iso) + 25569;
}

// Style indexes into cellXfs below.
const STYLE = { general: 0, bold: 1, date: 2, money: 3, integer: 4, percent: 5, isoDate: 6 };

const SHEET_STYLES = `${XML_HEADER}<styleSheet xmlns="${S}"><numFmts count="3"><numFmt numFmtId="164" formatCode="mmm d, yyyy"/><numFmt numFmtId="165" formatCode="&quot;$&quot;#,##0.00"/><numFmt numFmtId="166" formatCode="yyyy-mm-dd"/></numFmts><fonts count="2"><font><sz val="11"/><name val="Calibri"/></font><font><b/><sz val="11"/><name val="Calibri"/></font></fonts><fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills><borders count="1"><border><left/><right/><top/><bottom/><diagonal/></border></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs><cellXfs count="7"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/><xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0" applyFont="1"/><xf numFmtId="164" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="165" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="1" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="10" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/><xf numFmtId="166" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/></cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>`;

/// A cell value: a string, a number, null (empty), or an object
/// `{ text, bold }`, `{ date: iso, style: 'date'|'isoDate' }`,
/// `{ money: number }`, `{ number, style }`.
function sheetCell(reference, value, sharedStrings, useShared) {
  if (value === null || value === undefined || value === '') return '';
  const spec = typeof value === 'object' ? value : typeof value === 'number' ? { number: value } : { text: value };
  if (spec.date) return `<c r="${reference}" s="${STYLE[spec.style ?? 'date']}"><v>${excelSerial(spec.date)}</v></c>`;
  if (spec.money !== undefined) return `<c r="${reference}" s="${STYLE.money}"><v>${spec.money}</v></c>`;
  if (spec.number !== undefined) return `<c r="${reference}"${spec.style ? ` s="${STYLE[spec.style]}"` : ''}><v>${spec.number}</v></c>`;
  assertSupported(spec.text);
  const style = spec.bold ? ` s="${STYLE.bold}"` : '';
  if (useShared) {
    let index = sharedStrings.index.get(spec.text);
    if (index === undefined) {
      index = sharedStrings.list.length;
      sharedStrings.list.push(spec.text);
      sharedStrings.index.set(spec.text, index);
    }
    sharedStrings.count += 1;
    return `<c r="${reference}"${style} t="s"><v>${index}</v></c>`;
  }
  return `<c r="${reference}"${style} t="inlineStr"><is><t xml:space="preserve">${xml(spec.text)}</t></is></c>`;
}

/// A workbook. Sheets: `{ name, rows: [[cell...]], widths, sharedStrings }`.
/// Strings go to the shared string table unless a sheet sets
/// `sharedStrings: false`, in which case they are inline, as some exporters
/// write them.
export function xlsx({ sheets, title = '', creator = 'InternBench' }) {
  const shared = { list: [], index: new Map(), count: 0 };
  const sheetXml = sheets.map((sheet) => {
    const useShared = sheet.sharedStrings !== false;
    const rows = sheet.rows.map((row, rowIndex) => {
      const cells = row.map((value, columnIndex) => sheetCell(`${columnName(columnIndex)}${rowIndex + 1}`, value, shared, useShared)).join('');
      return cells ? `<row r="${rowIndex + 1}">${cells}</row>` : '';
    }).join('');
    const widths = sheet.widths ? `<cols>${sheet.widths.map((width, index) => `<col min="${index + 1}" max="${index + 1}" width="${width}" customWidth="1"/>`).join('')}</cols>` : '';
    const lastColumn = Math.max(1, ...sheet.rows.map((row) => row.length));
    const dimension = `A1:${columnName(lastColumn - 1)}${Math.max(1, sheet.rows.length)}`;
    return `${XML_HEADER}<worksheet xmlns="${S}" xmlns:r="${REL}"><dimension ref="${dimension}"/><sheetViews><sheetView workbookViewId="0"/></sheetViews><sheetFormatPr defaultRowHeight="15"/>${widths}<sheetData>${rows}</sheetData><pageMargins left="0.7" right="0.7" top="0.75" bottom="0.75" header="0.3" footer="0.3"/></worksheet>`;
  });
  const overrides = [
    ['/xl/workbook.xml', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml'],
    ['/xl/styles.xml', 'application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml'],
    ['/docProps/core.xml', 'application/vnd.openxmlformats-package.core-properties+xml'],
    ...sheets.map((_, index) => [`/xl/worksheets/sheet${index + 1}.xml`, 'application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml']),
    ...(shared.list.length ? [['/xl/sharedStrings.xml', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml']] : []),
  ];
  const workbookRels = [
    ...sheets.map((_, index) => [`rId${index + 1}`, 'worksheet', `worksheets/sheet${index + 1}.xml`]),
    [`rId${sheets.length + 1}`, 'styles', 'styles.xml'],
    ...(shared.list.length ? [[`rId${sheets.length + 2}`, 'sharedStrings', 'sharedStrings.xml']] : []),
  ];
  return zip([
    ['[Content_Types].xml', contentTypes(overrides)],
    ['_rels/.rels', `${XML_HEADER}<Relationships xmlns="${PKG_REL}"><Relationship Id="rId1" Type="${REL}/officeDocument" Target="xl/workbook.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/></Relationships>`],
    ['docProps/core.xml', coreProperties(title, creator)],
    ['xl/workbook.xml', `${XML_HEADER}<workbook xmlns="${S}" xmlns:r="${REL}"><bookViews><workbookView/></bookViews><sheets>${sheets.map((sheet, index) => `<sheet name="${xml(sheet.name)}" sheetId="${index + 1}" r:id="rId${index + 1}"/>`).join('')}</sheets></workbook>`],
    ['xl/_rels/workbook.xml.rels', relationships(workbookRels)],
    ['xl/styles.xml', SHEET_STYLES],
    ...sheetXml.map((content, index) => [`xl/worksheets/sheet${index + 1}.xml`, content]),
    ...(shared.list.length ? [['xl/sharedStrings.xml', `${XML_HEADER}<sst xmlns="${S}" count="${shared.count}" uniqueCount="${shared.list.length}">${shared.list.map((text) => `<si><t xml:space="preserve">${xml(text)}</t></si>`).join('')}</sst>`]] : []),
  ]);
}

/// How the worker renders one cell: a date serial as an ISO date, a number
/// the way Rust prints a float, text as is.
export function sheetCellText(value) {
  if (value === null || value === undefined) return '';
  if (typeof value === 'number') return String(value);
  if (typeof value === 'string') return value;
  if (value.date) return value.date;
  if (value.money !== undefined) return String(value.money);
  if (value.number !== undefined) return String(value.number);
  return value.text;
}

/// The text of each sheet as the worker renders it: `## name` and a pipe
/// table of the used range.
export function xlsxText(sheets) {
  return sheets.map((sheet) => [`## ${sheet.name}`, ...sheet.rows.map((row) => `| ${row.map(sheetCellText).join(' | ')} |`)].join('\n'));
}
