//! The engine reads the layouts this worker writes through mirror types of
//! its own. A kind, route or field one side has and the other lacks would
//! fail every document that used it, or quietly drop what it carried, so
//! the worker's every variant and every field goes through the real
//! protocol line into the engine and back, and must come back unchanged.

use intern_worker::extract::{ExtractedDocument, ExtractedPage, PageSource};
use intern_worker::layout::{
    BlockKind, KeyValue, LayoutBlock, LayoutCell, LayoutLine, LayoutRow, LayoutTable, PageLayout,
    PageRoute, RouteSignals, TextSource,
};
use intern_worker::protocol::{Event, Response};

const KINDS: [BlockKind; 9] = [
    BlockKind::Heading,
    BlockKind::Paragraph,
    BlockKind::ListItem,
    BlockKind::Table,
    BlockKind::KeyValue,
    BlockKind::Caption,
    BlockKind::PageHeader,
    BlockKind::PageFooter,
    BlockKind::Other,
];

const ROUTES: [PageRoute; 4] = [
    PageRoute::Fast,
    PageRoute::Layout,
    PageRoute::Ocr,
    PageRoute::OcrRegions,
];

/// Every signal set, each to a value of its own, so a signal the engine
/// does not know comes back as zero and is caught.
fn signals() -> RouteSignals {
    let mut value = serde_json::to_value(RouteSignals::default()).unwrap();
    for (index, (_, signal)) in value.as_object_mut().unwrap().iter_mut().enumerate() {
        *signal = serde_json::json!(index + 1);
    }
    serde_json::from_value(value).unwrap()
}

/// A block of `kind` with every optional field filled.
fn block(page: usize, number: usize, kind: BlockKind, source: TextSource) -> LayoutBlock {
    let id = format!("p{page}.b{number}");
    LayoutBlock {
        id: id.clone(),
        kind,
        text: format!("Block {number} of page {page}"),
        bbox: Some([10, 20 * number as u32, 400, 20 * number as u32 + 15]),
        level: Some(2),
        section: Some(format!("p{page}.b1")),
        source,
        confidence: Some(91),
        lines: vec![LayoutLine {
            text: format!("Block {number} of page {page}"),
            bbox: Some([10, 20 * number as u32, 400, 20 * number as u32 + 15]),
            confidence: Some(88),
        }],
        table: Some(LayoutTable {
            rows: vec![LayoutRow {
                id: format!("{id}.r1"),
                cells: vec![LayoutCell {
                    id: format!("{id}.r1.c1"),
                    text: "Invoice No.".into(),
                    bbox: Some([10, 20, 120, 35]),
                    header: true,
                }],
            }],
        }),
        fields: vec![KeyValue {
            id: format!("{id}.f1"),
            key: "Invoice date".into(),
            value: "May 1, 2026".into(),
            key_bbox: Some([10, 40, 90, 55]),
            value_bbox: Some([100, 40, 180, 55]),
        }],
    }
}

fn document() -> ExtractedDocument {
    let pages = ROUTES
        .iter()
        .enumerate()
        .map(|(index, route)| {
            let page = index + 1;
            let source = if matches!(route, PageRoute::Ocr) {
                TextSource::Ocr
            } else {
                TextSource::Native
            };
            let mut blocks = KINDS
                .iter()
                .enumerate()
                .map(|(number, kind)| block(page, number + 1, *kind, source))
                .collect::<Vec<_>>();
            // A reader without geometry: no boxes anywhere.
            blocks.push(LayoutBlock {
                id: format!("p{page}.b{}", KINDS.len() + 1),
                kind: BlockKind::Paragraph,
                text: "Plain".into(),
                bbox: None,
                level: None,
                section: None,
                source: TextSource::Ocr,
                confidence: None,
                lines: vec![LayoutLine {
                    text: "Plain".into(),
                    bbox: None,
                    confidence: None,
                }],
                table: None,
                fields: Vec::new(),
            });
            let text = blocks
                .iter()
                .map(|block| block.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");
            ExtractedPage {
                page_number: page,
                text,
                source: if matches!(route, PageRoute::Ocr) {
                    PageSource::Ocr
                } else {
                    PageSource::Native
                },
                ocr_confidence: matches!(route, PageRoute::Ocr).then_some(91.0),
                vision_escalated: false,
                layout: Some(PageLayout {
                    width: 6120,
                    height: 7920,
                    route: *route,
                    signals: signals(),
                    blocks,
                }),
            }
        })
        .collect();
    ExtractedDocument {
        pages,
        warnings: Vec::new(),
        truncated: false,
        optional_image: None,
        timings: None,
    }
}

#[test]
fn every_layout_the_worker_writes_reaches_the_engine_whole() {
    let sent = document();
    let line = serde_json::to_string(&Response::new(
        "contract",
        Event::Parsed {
            document: sent.clone(),
        },
    ))
    .unwrap();

    let response = intern_engine::worker::decode_worker_response(&line).unwrap();
    let intern_engine::worker::WorkerEvent::Parsed { document } = response.event else {
        panic!("not a parsed event: {line}");
    };
    let source = intern_engine::worker::adapt_document(document).unwrap();

    assert_eq!(source.pages.len(), sent.pages.len());
    for (received, sent) in source.pages.iter().zip(&sent.pages) {
        let layout = received.layout.as_ref().expect("the layout arrived");
        // Back into the worker's own type: whatever the engine dropped or
        // changed shows up as a difference.
        let back: PageLayout =
            serde_json::from_value(serde_json::to_value(layout).unwrap()).unwrap();
        assert_eq!(&back, sent.layout.as_ref().unwrap());
    }

    // And the engine can build its structured document from all of it.
    let structured = intern_engine::structure::structured(&source);
    assert!(structured.block("p4.b5").is_some());
}

/// A page the engine has no layout for - stored before layouts existed, or
/// built from plain text - is segmented by the engine exactly as this
/// worker segments a page of text it has no geometry for: the same blocks,
/// ids, kinds, text, lines, table rows and cells, labelled values and
/// sections. The evidence index reads either the same way.
#[test]
fn the_engine_segments_text_exactly_as_the_worker_does() {
    let long_paragraph = (0..30)
        .map(|line| {
            if line % 5 == 4 {
                format!("line {line} ends here.")
            } else {
                format!("line {line} runs on")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let documents: Vec<Vec<String>> = vec![
        vec![
            "# Written Consent\n\nThe undersigned directors consent.\nIt is resolved.\n\n## Resolutions\n\n| Item | Vote |\n| --- | --- |\n| Budget | For |\n|  | Against |\n\n- First point\n- Second point\nDate: March 4, 2026\nSigned by: Ada Example".to_owned(),
        ],
        vec![
            "NOTICE OF TERMINATION\nThe tenant will pay the following amounts: rent and fees.\nTime: 12:01 a.m.\nhttp://example.test/x".to_owned(),
            "PARTIES:\nLandlord: Oakhaven Retail Properties LLC\nTenant: Sorrel & Thistle Tea House LLC\n\n(a) first item\n(iv) fourth item\n3) third\n• bullet\n–dash\n- \n\n| Esc \\| aped | Cell |\n| :---: | --- |\n| 1 | 2 |".to_owned(),
        ],
        vec![long_paragraph],
        vec![
            "INVOICE\r\nInvoice Number: INV-20417\r\nInvoice Date: 03/04/2026\r\n\r\nBill To:\r\nQuillon Ridge Bakery, Inc.\r\n   \r\n  indented line\r\n####### not a heading\r\n#\r\n# ".to_owned(),
            String::new(),
            "\n\n\n".to_owned(),
            "ARTICLE 2. RENT\nTenant pays rent monthly.\nIn advance.\n\n| Year | Rent |\n| --- | --- |\n| 1 | $34.00 |\n\nSigned.".to_owned(),
        ],
    ];
    for pages in documents {
        let mut layouts = pages
            .iter()
            .enumerate()
            .map(|(index, text)| PageLayout::of_text(index + 1, text))
            .collect::<Vec<_>>();
        intern_worker::layout::assign_sections(layouts.iter_mut());
        let source = intern_engine::DocumentSource::from_pages(
            pages
                .iter()
                .enumerate()
                .map(|(index, text)| {
                    intern_engine::SourcePage::new(
                        index + 1,
                        text.clone(),
                        intern_engine::PageOrigin::Native,
                    )
                })
                .collect(),
        );
        let structured = intern_engine::structure::structured(&source);
        assert_eq!(structured.pages.len(), layouts.len());
        for (engine_page, worker_page) in structured.pages.iter().zip(&layouts) {
            let engine = serde_json::to_value(&engine_page.blocks).unwrap();
            let worker = serde_json::to_value(&worker_page.blocks).unwrap();
            assert_eq!(engine, worker, "{pages:?}");
        }
    }
}
