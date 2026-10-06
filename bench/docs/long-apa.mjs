/// A twenty-five-page asset purchase agreement for a specialty food maker:
/// eleven articles and the seller's disclosure schedules, which list dozens
/// of other agreements, registrations, leases, and permits, each with its
/// own date.
import { Flow } from '../lib/layout.mjs';
import { Rng } from '../lib/rng.mjs';
import { gold } from '../lib/gold.mjs';
import { addDays, amount, longDate, money, numericDate } from '../lib/format.mjs';
import { companies, people } from '../lib/names.mjs';
import { digitalPdf, result, signatureBlocks } from './common.mjs';
import { coverPage, tableOfContents, writeArticles } from './agreement.mjs';

export function assetPurchaseAgreement() {
  const id = 'asset-purchase-agreement-25p';
  const rng = Rng.from(id);
  const buyer = 'Kingsfold Specialty Foods Inc.';
  const seller = 'Bramblecrest Preserves LLC';
  const members = ['Calla Bramble', 'Thaddeus Bramble'];
  const escrowAgent = 'Cobble Hill Trust Company';
  const dated = '2026-03-30';
  const closing = '2026-04-30';
  const outside = '2026-06-30';
  const balanceSheet = '2025-12-31';
  const price = 1425000000;
  const escrow = 142500000;
  const target = 318000000;
  const flow = new Flow({
    face: 'serif', fontSize: 12, leading: 1.45, margins: { top: 72, bottom: 72 }, keep: [buyer, seller],
    footer: (page, { number }) => {
      if (number > 1) page.textCenter(306, 760, `Asset Purchase Agreement - ${number}`, { face: 'sans', size: 7.5, grey: 0.4 });
    },
  });
  coverPage(flow, {
    title: 'ASSET PURCHASE AGREEMENT',
    between: [
      { text: 'by and between' },
      { text: buyer.toUpperCase(), bold: true },
      { text: 'as Buyer,' },
      { text: 'and' },
      { text: seller.toUpperCase(), bold: true },
      { text: 'as Seller' },
    ],
    dated: `Dated as of ${longDate(dated)}`,
    footer: 'CONFIDENTIAL - subject to the confidentiality agreement between the parties',
  });

  const contractCounterparties = companies(rng.fork('counterparties'), 14, { exclude: [buyer, seller] });
  const articles = [
    ['Definitions', [
      ['Defined Terms', 'In this Agreement the following terms have the meanings below; other terms are defined where they first appear.', [
        '"Business" means the business of developing, manufacturing, marketing, and selling fruit preserves, savory jams, fruit butters, and pie fillings under the Bramblecrest and Hollow Orchard brands, as conducted by Seller on the date of this Agreement.',
        `"Closing Working Capital" means current assets of the Business (inventory, trade receivables, and prepaid expenses) minus current liabilities of the Business (trade payables and accrued expenses) assumed by Buyer, determined as of 11:59 p.m. on the day before the Closing Date in accordance with the Accounting Principles, and "Target Working Capital" means ${money(target)}.`,
        '"Accounting Principles" means GAAP applied using the same accounting methods, policies, practices, and estimation methodologies used in preparing the Balance Sheet, as illustrated in Schedule 1.1.',
        `"Balance Sheet" means the reviewed balance sheet of the Business as of ${longDate(balanceSheet)}, and "Balance Sheet Date" means that date.`,
        '"Knowledge of Seller" means the actual knowledge, after reasonable inquiry of their direct reports, of Calla Bramble, Thaddeus Bramble, and the plant manager of the Seller\'s Briarport facility.',
        '"Material Adverse Effect" means any event or change that is materially adverse to the Purchased Assets or the results of operations or condition of the Business, taken as a whole, other than changes in general economic conditions, in the specialty food industry generally, in law or GAAP, or resulting from the announcement of this Agreement, except to the extent they affect the Business disproportionately.',
        '"Permitted Encumbrances" means liens for taxes not yet due, statutory liens of landlords, carriers, and warehousemen for amounts not yet delinquent, and the matters listed in Schedule 1.2.',
      ]],
    ]],
    ['Purchase and Sale', [
      ['Purchased Assets', 'At the Closing, Seller will sell, assign, and deliver to Buyer, and Buyer will purchase, free and clear of all encumbrances other than Permitted Encumbrances, all of Seller\'s right, title, and interest in the assets used in the Business (the "Purchased Assets"), including:', [
        'all inventory of finished goods, work in process, fruit and sugar, jars, lids, labels, and cartons;',
        'all trade accounts receivable of the Business outstanding at the Closing;',
        'the contracts listed in Schedule 2.1(c) (the "Assigned Contracts");',
        'the trademarks, recipes, formulas, and domain names listed in Schedule 5.8, with the goodwill associated with them;',
        'the machinery, equipment, tooling, and vehicles listed in Schedule 5.10;',
        'the real property leases listed in Schedule 5.9 and the leasehold improvements at those premises;',
        'all permits held by Seller for the Business, to the extent transferable, as listed in Schedule 5.15;',
        'all customer lists, supplier lists, price lists, marketing materials, and product photography; and',
        'all books and records relating to the Business, other than those described in Section 2.2.',
      ]],
      ['Excluded Assets', 'The Purchased Assets do not include: cash and cash equivalents; Seller\'s bank accounts; Seller\'s organizational records and tax returns; insurance policies and the right to any claims under them except as provided in Section 7.6; the farmhouse property at 44 Juniper Loop owned by the Members; and any rights of Seller under this Agreement.'],
      ['Assumed Liabilities', 'Buyer will assume only (a) trade payables of the Business included in Closing Working Capital, (b) obligations under the Assigned Contracts arising after the Closing, other than those arising from a breach before the Closing, and (c) accrued vacation of Transferred Employees to the extent included in Closing Working Capital (the "Assumed Liabilities").'],
      ['Excluded Liabilities', 'Buyer will not assume any other liability of Seller, including liabilities for taxes of Seller or of the Business for any period before the Closing, indebtedness for borrowed money, liabilities to the Members, product liability claims for products sold before the Closing, and liabilities under any employee benefit plan of Seller.'],
    ]],
    ['Purchase Price', [
      ['Purchase Price', `The purchase price for the Purchased Assets is ${money(price)} (the "Base Price"), plus or minus the amount by which Closing Working Capital exceeds or falls short of Target Working Capital, plus any Earn-Out Payments, and plus the assumption of the Assumed Liabilities.`],
      ['Payment at Closing', `At the Closing Buyer will pay the Base Price, adjusted by Seller's good-faith estimate of Closing Working Capital, by wire transfer: ${money(escrow)} to ${escrowAgent} as escrow agent, to be held under the Escrow Agreement as security for Seller's indemnification obligations for eighteen months, and the balance to Seller.`],
      ['Working Capital Adjustment', 'Within ninety days after the Closing, Buyer will deliver its calculation of Closing Working Capital. Seller has forty-five days to object. Disputes not resolved within thirty days after an objection will be decided by an independent accounting firm whose fees are shared in inverse proportion to the parties\' success. Any adjustment of more than $25,000 is paid within five business days after it is final.'],
      ['Earn-Out', 'Buyer will pay Seller an earn-out payment for each of the first three fiscal years after the Closing in which net sales of products under the Bramblecrest brand meet the thresholds below. Buyer will operate the Business in good faith and will not take any action with the principal purpose of avoiding an earn-out payment.', [
        'Fiscal 2026: net sales of at least $21,500,000 - earn-out payment of $500,000;',
        'Fiscal 2027: net sales of at least $24,000,000 - earn-out payment of $750,000;',
        'Fiscal 2028: net sales of at least $27,000,000 - earn-out payment of $1,000,000.',
      ]],
      ['Allocation', 'The Base Price and any other consideration will be allocated among the Purchased Assets as set out in Schedule 3.4, and the parties will file all tax returns, including IRS Form 8594, consistently with that allocation.'],
    ]],
    ['Closing', [
      ['Closing', `The closing of the transactions contemplated by this Agreement (the "Closing") will take place remotely by exchange of documents on ${longDate(closing)}, or, if the conditions in Article VIII have not then been satisfied or waived, on the third business day after they are, or on another date the parties agree (the "Closing Date").`],
      ['Seller Deliverables', 'At the Closing Seller will deliver a bill of sale, an assignment and assumption agreement, assignments of the trademarks in Schedule 5.8 in recordable form, the third-party consents marked with an asterisk in Schedule 2.1(c), payoff letters and lien releases for all indebtedness secured by the Purchased Assets, the Escrow Agreement signed by Seller, and a certificate of its officer that the conditions in Section 8.2 are satisfied.'],
      ['Buyer Deliverables', 'At the Closing Buyer will deliver the payments required by Section 3.2, the assignment and assumption agreement, the Escrow Agreement signed by Buyer, and a certificate of its officer that the conditions in Section 8.3 are satisfied.'],
    ]],
    ['Representations and Warranties of Seller', [
      ['Organization', 'Seller is a limited liability company duly organized, validly existing, and in good standing under the laws of North Carolina, and is qualified to do business in each jurisdiction where the conduct of the Business requires it, all of which are listed in Schedule 5.1.'],
      ['Authority', 'Seller has full power and authority to enter into this Agreement and the other transaction documents and to perform its obligations under them. The Members have approved this Agreement, and no other action of Seller is required. This Agreement is a valid and binding obligation of Seller, enforceable in accordance with its terms.'],
      ['No Conflicts; Consents', 'The execution and performance of this Agreement by Seller do not violate its organizational documents or any law applicable to it, and, except for the consents listed in Schedule 2.1(c), do not require any consent under, or result in a breach of, any Assigned Contract.'],
      ['Financial Statements', `Schedule 5.4 contains the reviewed financial statements of the Business for fiscal years 2024 and 2025 and the Balance Sheet. They were prepared in accordance with GAAP consistently applied and fairly present the financial condition and results of operations of the Business as of their dates and for the periods they cover. Net sales of the Business were ${money(1912430000)} in fiscal 2025 and ${money(1744815000)} in fiscal 2024.`],
      ['Absence of Changes', `Since the Balance Sheet Date the Business has been conducted in the ordinary course, there has been no Material Adverse Effect, and Seller has not sold any assets of the Business other than inventory in the ordinary course, increased the compensation of any employee by more than four percent, or entered into any contract that would be a Material Contract.`],
      ['Title to Assets', 'Seller has good and valid title to, or a valid leasehold interest in, all the Purchased Assets, free and clear of encumbrances other than Permitted Encumbrances. The Purchased Assets are all the assets necessary to conduct the Business as currently conducted.'],
      ['Material Contracts', 'Schedule 5.7 lists every contract of the Business involving more than $50,000 a year, every contract with a customer or supplier listed in Schedule 5.17, every co-packing, licensing, distribution, and broker agreement, and every contract containing exclusivity or most-favored-customer terms. Each is valid and in full force, and neither Seller nor, to Seller\'s Knowledge, any counterparty is in material breach of it.'],
      ['Intellectual Property', 'Schedule 5.8 lists all registered trademarks and applications, domain names, and material unregistered marks used in the Business. Seller owns them free of encumbrances. To Seller\'s Knowledge the operation of the Business does not infringe the intellectual property of any person, and no person is infringing the Business\'s trademarks. Seller\'s recipes and formulas have been kept confidential and are disclosed only to employees and co-packers bound by confidentiality obligations.'],
      ['Real Property', 'Seller owns no real property used in the Business. Schedule 5.9 lists each lease of real property used in the Business. Each lease is in full force and effect, Seller is not in default under any of them, and Seller has received no notice of any condemnation affecting the leased premises.'],
      ['Condition of Equipment', 'The equipment in Schedule 5.10 is in good operating condition and repair, ordinary wear and tear excepted, and is adequate for the uses to which it is being put.'],
      ['Inventory and Receivables', 'Inventory consists of a quality and quantity usable and saleable in the ordinary course, except for obsolete items written down on the Balance Sheet; no finished goods are within ninety days of their best-by date in excess of normal levels. Receivables arose from bona fide sales and are collectible in the ordinary course, net of the reserves on the Balance Sheet.'],
      ['Employees', 'Schedule 5.12 lists each employee of the Business with title, hire date, and current rate of pay. Seller has complied in all material respects with employment and wage-and-hour laws. No employee is represented by a union, and there is no pending or, to Seller\'s Knowledge, threatened labor dispute.'],
      ['Employee Benefits', 'Schedule 5.13 lists each benefit plan of Seller. Each has been maintained in compliance with its terms and applicable law, and no liability under any of them will become a liability of Buyer.'],
      ['Taxes', 'Seller has filed all tax returns required of it and paid all taxes shown as due. There are no tax liens on the Purchased Assets other than for taxes not yet due, and no audit of Seller is pending.'],
      ['Litigation', 'Except as described in Schedule 5.15, there is no action pending or, to Seller\'s Knowledge, threatened against Seller relating to the Business or the Purchased Assets, and no order of any court binds the Business.'],
      ['Permits; Food Safety', 'Schedule 5.15 lists the permits, registrations, and certifications held for the Business, each of which is in full force. The Business has been operated in material compliance with food safety laws, has had no product recall or market withdrawal in the last five years, and its last third-party food safety audit resulted in a score of 96.5 out of 100.'],
      ['Customers and Suppliers', 'Schedule 5.17 lists the ten largest customers and the ten largest suppliers of the Business for fiscal 2025 by dollar volume. None of them has notified Seller that it intends to stop or materially reduce its business with the Business.'],
      ['Insurance', 'Schedule 5.18 lists the insurance policies covering the Business. Each is in full force, all premiums due have been paid, and there are no open claims other than those listed.'],
      ['Environmental Matters', 'Seller has complied in all material respects with environmental laws in operating the Business, holds the environmental permits listed in Schedule 5.15, and has not received any notice of violation or of any release of hazardous materials at the leased premises. Seller has delivered to Buyer the Phase I environmental site assessment of the Briarport plant summarized in Schedule 5.16, and has no Knowledge of any condition that report does not disclose.'],
      ['Product Warranty and Complaints', 'Each product sold by the Business conformed in all material respects to its label, specifications, and applicable law. Schedule 5.20 lists every consumer complaint involving foreign material, illness, or injury received in the last three years and how it was resolved. The Business gives no warranty beyond its standard satisfaction guarantee printed on each jar.'],
      ['Compliance with Laws', 'Seller has conducted the Business in compliance in all material respects with all applicable laws, including food labeling, allergen, weights and measures, and nutrition labeling laws, and has received no written notice of any violation that remains unresolved.'],
      ['Privacy', 'The Business collects personal information only through its online store and retail shop loyalty program, covering about 41,000 consumers. Seller has complied with its published privacy policy and applicable privacy laws, and has not experienced any security breach involving that information.'],
      ['Affiliate Transactions', 'Except as listed in Schedule 5.19, neither the Members nor any of their relatives or affiliates is party to any contract with the Business, owns any asset used in it, or has any claim against it.'],
      ['Books and Records', 'The books of account and other records of the Business are complete and correct in all material respects, have been maintained in accordance with sound business practices, and will be delivered to Buyer at the Closing.'],
      ['Brokers', 'No broker or finder is entitled to any fee from Seller in connection with this Agreement except Pemberly Falls Capital Advisors LLC, whose fee Seller will pay.'],
    ]],
    ['Representations and Warranties of Buyer', [
      ['Organization and Authority', 'Buyer is a corporation duly organized and in good standing under the laws of Delaware and has full corporate power to enter into and perform this Agreement, which has been duly authorized by all necessary corporate action and is a valid and binding obligation of Buyer.'],
      ['Financing', 'Buyer has, and at the Closing will have, sufficient funds available to pay the Base Price and all other amounts it is required to pay under this Agreement. Buyer\'s obligations are not conditioned on obtaining financing.'],
      ['Independent Investigation', 'Buyer has conducted its own investigation of the Business and acknowledges that, except for the representations in Article V, Seller makes no representation or warranty about the Business.'],
    ]],
    ['Covenants', [
      ['Conduct Before Closing', 'Until the Closing Seller will operate the Business in the ordinary course, maintain the Purchased Assets in good condition, keep available the services of its employees, and preserve its relationships with customers and suppliers, and will not sell or encumber any Purchased Asset other than inventory sold in the ordinary course.'],
      ['Access', 'Until the Closing Seller will give Buyer and its representatives reasonable access during business hours to the premises, books, records, and personnel of the Business, including for an environmental site assessment at the Briarport plant.'],
      ['Exclusivity', 'Until the earlier of the Closing and termination of this Agreement, neither Seller nor the Members will solicit, negotiate, or accept any proposal to acquire the Business or a material part of its assets.'],
      ['Employees', 'Buyer will offer employment, effective on the Closing Date, to each employee listed in Schedule 5.12 at no less than the same base pay, and will credit their service with Seller for vacation accrual and benefit plan eligibility. Employees who accept are "Transferred Employees".'],
      ['Non-Competition', 'For five years after the Closing, Seller and the Members will not engage anywhere in the United States in the manufacture or sale of fruit preserves, jams, or pie fillings, or solicit any customer or Transferred Employee of the Business. The Members agree to this covenant by signing the Joinder attached to this Agreement.'],
      ['Insurance Claims', 'After the Closing, Seller will pursue for Buyer\'s benefit any claim under Seller\'s insurance policies for loss of or damage to Purchased Assets occurring before the Closing and will pay any proceeds to Buyer.'],
      ['Transition', 'For six months after the Closing Seller will provide the transition services described in the Transition Services Agreement, including the use of Seller\'s enterprise resource planning system until Buyer migrates the Business to its own.'],
      ['Receivables and Wrong Pockets', 'Any payment Seller receives after the Closing for a Purchased Asset, including a receivable, will be held in trust for Buyer and remitted within ten business days. Any asset that should have been an Excluded Asset but is transferred to Buyer will be returned to Seller on request.'],
      ['Use of Name', 'Within thirty days after the Closing, Seller will change its legal name to one that does not include "Bramblecrest" or any confusingly similar name, and will file the change with the North Carolina Secretary of State.'],
      ['Notification of Developments', 'Until the Closing, each party will promptly notify the other of any event that would cause a representation of that party to be untrue in any material respect or a condition to Closing not to be satisfied. Notification does not cure any breach.'],
      ['Further Assurances', 'After the Closing each party will execute and deliver any further instruments, and take any further actions, that the other reasonably requests to vest the Purchased Assets in Buyer and to carry out this Agreement.'],
      ['Confidentiality', 'After the Closing, Seller and the Members will keep confidential all information about the Business, including its recipes, customer terms, and supplier pricing, except as required by law, and will not use it for any purpose.'],
    ]],
    ['Conditions to Closing', [
      ['Conditions of Both Parties', 'No order of any court restraining the transactions is in effect, and any waiting period under applicable antitrust laws has expired or been terminated.'],
      ['Conditions of Buyer', 'Seller\'s representations are true in all material respects at the Closing, Seller has performed its covenants in all material respects, no Material Adverse Effect has occurred, the consents marked with an asterisk in Schedule 2.1(c) have been obtained, and Calla Bramble has signed a consulting agreement with Buyer for twelve months.'],
      ['Conditions of Seller', 'Buyer\'s representations are true in all material respects at the Closing and Buyer has performed its covenants in all material respects.'],
    ]],
    ['Indemnification', [
      ['Survival', 'The representations of the parties survive the Closing for eighteen months, except that the representations in Sections 5.1, 5.2, 5.6, and 5.14 survive until sixty days after the applicable statute of limitations expires.'],
      ['Indemnification by Seller', 'Seller will indemnify Buyer against losses arising from any breach of Seller\'s representations or covenants, any Excluded Asset, and any Excluded Liability.'],
      ['Indemnification by Buyer', 'Buyer will indemnify Seller against losses arising from any breach of Buyer\'s representations or covenants and any Assumed Liability.'],
      ['Limitations', `Seller is not liable for breaches of representations (other than those surviving to the statute of limitations, and fraud) until losses exceed ${money(14250000)} in the aggregate, and then only for the excess, and its total liability for those breaches will not exceed the escrow amount of ${money(escrow)}. Claims are first satisfied from the escrow.`],
      ['Exclusive Remedy', 'After the Closing, this Article is the parties\' exclusive remedy for any claim arising out of this Agreement, other than claims for fraud, equitable relief, or the working capital adjustment.'],
    ]],
    ['Termination', [
      ['Termination', `This Agreement may be terminated before the Closing by mutual written consent; by either party if the Closing has not occurred by ${longDate(outside)} (the "Outside Date"), unless the terminating party's breach caused the delay; or by either party if the other materially breaches this Agreement and does not cure the breach within twenty days after notice.`],
      ['Effect of Termination', 'If this Agreement is terminated, it becomes void without liability of either party, except for liability for willful breach before termination and except that the confidentiality obligations and this Article survive.'],
    ]],
    ['Miscellaneous', [
      ['Expenses', 'Each party pays its own expenses, except that Buyer and Seller share equally the fees of the escrow agent and any transfer taxes.'],
      ['Notices', `Notices must be in writing and are effective on delivery by hand or nationally recognized overnight courier to: if to Buyer, ${buyer}, 900 Kingsfold Avenue, Westharrow, MI 49101, Attention: General Counsel; if to Seller, ${seller}, 1410 Orchard Row, Briarport, NC 27514, Attention: Calla Bramble.`],
      ['Entire Agreement; Amendment', 'This Agreement, its schedules, and the other transaction documents are the entire agreement of the parties on their subject matter and supersede the letter of intent between them. This Agreement may be amended only in a writing signed by both parties.'],
      ['Governing Law; Venue', 'This Agreement is governed by Delaware law. Each party submits to the exclusive jurisdiction of the state and federal courts located in Wilmington, Delaware, and waives trial by jury.'],
      ['Counterparts', 'This Agreement may be executed in counterparts and delivered electronically, each of which is an original and all of which together are one instrument.'],
    ]],
  ];
  const schedules = ['Schedule 1.1 Accounting Principles', 'Schedule 2.1(c) Assigned Contracts', 'Schedule 3.4 Allocation', 'Schedule 5.7 Material Contracts', 'Schedule 5.8 Intellectual Property', 'Schedule 5.9 Leased Real Property', 'Schedule 5.10 Equipment', 'Schedule 5.12 Employees', 'Schedule 5.15 Permits and Litigation', 'Schedule 5.17 Customers and Suppliers', 'Schedule 5.18 Insurance'];
  tableOfContents(flow, articles, schedules);
  flow.pageBreak();
  flow.heading('ASSET PURCHASE AGREEMENT', { level: 1, align: 'center' });
  flow.paragraph(`This Asset Purchase Agreement (this "Agreement") is dated as of ${longDate(dated)} and is between ${buyer}, a Delaware corporation ("Buyer"), and ${seller}, a North Carolina limited liability company ("Seller"). ${members[0]} and ${members[1]}, the members of Seller (the "Members"), join this Agreement solely for the purposes of Section 7.5.`);
  flow.paragraph('Seller makes and sells fruit preserves and related products from its plant in Briarport, North Carolina. Seller wishes to sell, and Buyer wishes to buy, substantially all of the assets of the Business, on the terms of this Agreement. The parties therefore agree as follows:');
  writeArticles(flow, articles);
  flow.paragraph('IN WITNESS WHEREOF, the parties have executed this Asset Purchase Agreement as of the date first written above.', { before: 8 });
  signatureBlocks(flow, [
    { heading: 'BUYER', entity: buyer.toUpperCase(), name: 'Leopold Haverford', title: 'Chief Executive Officer' },
    { heading: 'SELLER', entity: seller.toUpperCase(), name: members[0], title: 'Managing Member' },
  ]);
  flow.paragraph('JOINDER OF MEMBERS. Each of the undersigned joins this Agreement for the purposes of Section 7.5 (Non-Competition) only.', { face: 'sans-bold', size: 9 });
  signatureBlocks(flow, [{ name: members[0] }, { name: members[1] }], { size: 9.5 });

  // Disclosure schedules: data with dates, numbers, and names on every line.
  flow.pageBreak();
  flow.heading('SELLER DISCLOSURE SCHEDULES', { level: 1, align: 'center' });
  flow.paragraph('These schedules are delivered by Seller under the Asset Purchase Agreement. Section numbers refer to the Agreement. Disclosure in any schedule applies to any other section to which its relevance is reasonably apparent.', { size: 9.5 });
  const contractTitles = ['Co-Packing Agreement', 'Supply Agreement (strawberries)', 'Supply Agreement (glass jars)', 'Broker Agreement - Southeast region', 'Distribution Agreement', 'Private Label Supply Agreement', 'Cold Storage Warehouse Agreement', 'Freight Services Agreement', 'Label Printing Agreement', 'Equipment Lease (filler line 2)', 'Software Subscription (ERP)', 'Trademark Coexistence Agreement', 'Sugar Supply Agreement', 'Waste Hauling Agreement'];
  const contractRng = rng.fork('contracts');
  const contracts = contractTitles.map((title, index) => {
    const start = addDays('2019-01-15', contractRng.int(0, 2400));
    const term = contractRng.pick(['1 year, auto-renews', '2 years', '3 years', '5 years', 'Until terminated on 90 days notice']);
    const value = contractRng.amount(6000000, 240000000, 500000);
    return [`${index + 1}${index % 4 === 0 ? '*' : ''}`, contractCounterparties[index], title, longDate(start), term, money(value, { cents: false })];
  });
  flow.heading('Schedule 2.1(c) and 5.7 - Assigned Contracts and Material Contracts', { level: 3 });
  flow.paragraph('An asterisk marks a contract whose assignment requires the counterparty\'s consent.', { size: 8.5 });
  flow.table([{ header: 'No.', width: 0.06 }, { header: 'Counterparty', width: 0.27 }, { header: 'Contract', width: 0.24 }, { header: 'Dated', width: 0.15 }, { header: 'Term', width: 0.15 }, { header: 'Annual value', width: 0.13, align: 'right' }], contracts, { size: 8 });
  flow.heading('Schedule 5.7(b) - Principal Terms of Key Contracts', { level: 3 });
  const termsRng = rng.fork('contract-terms');
  contracts.slice(0, 9).forEach(([number, counterparty, title, date, term, value]) => {
    const notice = termsRng.pick([30, 60, 90, 120]);
    const clause = {
      'Co-Packing Agreement': `${counterparty} manufactures the Hollow Orchard fruit butters to Seller's recipes at its own plant; Seller supplies fruit and jars; tolling fee of $${termsRng.int(4, 9)}.${termsRng.int(10, 95)} per case; minimum of ${termsRng.int(8, 30) * 1000} cases a year`,
      'Supply Agreement (strawberries)': `fixed price of $${termsRng.int(68, 99)} per hundredweight for IQF strawberries for the ${termsRng.pick(['2026', '2026 and 2027'])} crop years, with a ${termsRng.int(10, 25)}% volume flex either way`,
      'Supply Agreement (glass jars)': `annual volume commitment of ${termsRng.int(2, 6)}.${termsRng.int(0, 9)} million jars across three sizes; price adjusts each January by a natural gas surcharge index`,
      'Broker Agreement - Southeast region': `commission of ${termsRng.int(3, 6)}% of net invoiced sales to grocery chains in nine southeastern states; broker may not represent competing preserves brands`,
      'Distribution Agreement': `non-exclusive distribution to independent grocers and specialty retailers; distributor margin of ${termsRng.int(18, 28)}%; quarterly promotional calendar agreed in advance`,
      'Private Label Supply Agreement': `supply of three preserves under the customer's store brand at cost-plus pricing (cost plus ${termsRng.int(12, 22)}%); customer owns the label artwork, Seller owns the recipes`,
      'Cold Storage Warehouse Agreement': `up to ${termsRng.int(300, 900)} pallet positions at 34 to 38 F for fruit; storage $${termsRng.int(14, 26)} per pallet per month plus handling`,
      'Freight Services Agreement': `less-than-truckload rates at a ${termsRng.int(38, 62)}% discount off the carrier's base tariff, with a fuel surcharge table; claims filed within nine months`,
      'Label Printing Agreement': `pressure-sensitive labels for all retail SKUs; plate charges waived for orders above ${termsRng.int(20, 80)} thousand labels; 15 business day lead time`,
    }[title];
    flow.paragraph(`Contract ${number.replace('*', '')} - ${title} with ${counterparty}, dated ${date} (${term}, approximately ${value} a year): ${clause}. Either party may terminate on ${notice} days' notice after the initial term.${number.includes('*') ? ' Assignment requires the counterparty\'s consent, which Seller has requested.' : ''}`, { size: 10 });
  });
  flow.heading('Schedule 3.4 - Allocation of Purchase Price', { level: 3 });
  const allocation = [['Inventory', 286400000], ['Accounts receivable', 197250000], ['Machinery and equipment', 311800000], ['Leasehold improvements', 64300000], ['Trademarks and recipes', 382900000], ['Customer relationships', 121500000], ['Non-competition covenant', 25000000]];
  const goodwill = price - allocation.reduce((sum, [, value]) => sum + value, 0);
  flow.table([{ header: 'Asset class', width: 0.6 }, { header: 'Allocated amount', width: 0.4, align: 'right' }], [...allocation.map(([label, value]) => [label, money(value)]), ['Goodwill (residual)', money(goodwill)], [{ text: 'Base Price', face: 'sans-bold' }, { text: money(price), face: 'sans-bold' }]], { size: 8.5 });
  flow.heading('Schedule 5.4 - Financial Statements (reviewed)', { level: 3 });
  flow.table([{ header: 'Statement of income (in dollars)', width: 0.52 }, { header: 'Fiscal 2025', width: 0.24, align: 'right' }, { header: 'Fiscal 2024', width: 0.24, align: 'right' }], [
    ['Net sales', amount(1912430000), amount(1744815000)],
    ['Cost of goods sold', `(${amount(1290890000)})`, `(${amount(1196940000)})`],
    ['Gross profit', amount(621540000), amount(547875000)],
    ['Selling and marketing', `(${amount(201670000)})`, `(${amount(187420000)})`],
    ['General and administrative', `(${amount(148210000)})`, `(${amount(139960000)})`],
    ['Depreciation', `(${amount(61840000)})`, `(${amount(58310000)})`],
    ['Operating income', amount(209820000), amount(162185000)],
    ['Interest expense', `(${amount(14920000)})`, `(${amount(17345000)})`],
    ['Net income (pass-through entity; no income tax provision)', amount(194900000), amount(144840000)],
  ], { size: 8.5 });
  flow.table([{ header: `Balance sheet as of ${longDate(balanceSheet)} (in dollars)`, width: 0.6 }, { header: 'Amount', width: 0.4, align: 'right' }], [
    ['Cash', amount(48210000)], ['Trade receivables, net', amount(197250000)], ['Inventories', amount(286400000)], ['Prepaid expenses', amount(9400000)],
    ['Property and equipment, net', amount(376100000)], ['Total assets', amount(917360000)],
    ['Trade payables', amount(141200000)], ['Accrued expenses', amount(33850000)], ['Line of credit (Kingsfold Community Bank)', amount(90000000)], ['Equipment loans', amount(118400000)], ['Members\' equity', amount(533910000)], ['Total liabilities and members\' equity', amount(917360000)],
  ], { size: 8.5 });
  flow.heading('Schedule 5.8 - Intellectual Property', { level: 3 });
  const marks = [['BRAMBLECREST', '4,118,392', '2012-03-27', 'Classes 29, 30'], ['BRAMBLECREST (and design)', '4,559,018', '2014-07-01', 'Class 29'], ['HOLLOW ORCHARD', '5,302,774', '2017-10-03', 'Class 29'], ['PIE NIGHT', '5,881,240', '2019-09-24', 'Class 30'], ['SMALL KETTLE, BIG FRUIT', '6,204,915', '2020-12-08', 'Classes 29, 35'], ['RIDGE RUN RED', 'Application 97/612,404', '2024-11-12 (filed)', 'Class 29']];
  flow.table([{ header: 'Mark', width: 0.34 }, { header: 'Registration', width: 0.24 }, { header: 'Date', width: 0.2 }, { header: 'Classes', width: 0.22 }], marks.map(([mark, number, date, classes]) => [mark, number, /^\d{4}-\d\d-\d\d$/.test(date) ? longDate(date) : date, classes]), { size: 8.5 });
  flow.paragraph('Domain names: bramblecrest.example, hollow-orchard.example, pienight.example. Unregistered marks: "Kettle Batch", "Orchard Reserve". Recipes and formulas: 64 active product formulas held in the Seller\'s recipe management system, including the 1987 strawberry rhubarb base recipe; none is licensed to any third party except co-packer use under contract 1.', { size: 8.5 });
  flow.heading('Schedule 5.9 - Leased Real Property', { level: 3 });
  flow.table([{ header: 'Premises', width: 0.32 }, { header: 'Landlord', width: 0.26 }, { header: 'Lease dated', width: 0.14 }, { header: 'Expires', width: 0.14 }, { header: 'Sq. ft.', width: 0.14, align: 'right' }], [
    ['1410 Orchard Row, Briarport, NC (plant and offices)', 'Orchard Row Industrial LLC', longDate('2016-05-01'), longDate('2031-04-30'), '48,200'],
    ['22 Pennant Street, Briarport, NC (finished goods warehouse)', 'Pennant Logistics Park LLC', longDate('2021-09-15'), longDate('2026-09-14'), '31,500'],
    ['3 Market Square, Briarport, NC (retail shop)', 'Market Square Partners', longDate('2023-03-01'), longDate('2028-02-29'), '1,850'],
  ], { size: 8.5 });
  flow.heading('Schedule 5.10 - Equipment', { level: 3 });
  const equipmentRng = rng.fork('equipment');
  const equipment = ['Steam-jacketed kettle, 300 gal', 'Steam-jacketed kettle, 300 gal', 'Steam-jacketed kettle, 150 gal', 'Vacuum cooker, 500 gal', 'Piston filler line 1 (24 head)', 'Piston filler line 2 (12 head, leased)', 'Capper, chuck type', 'Pressure-sensitive labeler', 'Pasteurization tunnel, 40 ft', 'Case erector and sealer', 'Metal detector', 'Walk-in cooler compressor set', 'Forklift, electric, 5,000 lb', 'Forklift, electric, 3,000 lb', 'Delivery truck, 26 ft box', 'Boiler, 150 hp'];
  flow.table([{ header: 'Item', width: 0.42 }, { header: 'Serial no.', width: 0.2 }, { header: 'Year', width: 0.1, align: 'right' }, { header: 'Net book value', width: 0.28, align: 'right' }], equipment.map((item) => [item, `${['KTL', 'VC', 'FL', 'CP', 'LB', 'PT', 'CE', 'MD', 'CR', 'FK', 'TR', 'BL'][equipmentRng.int(0, 11)]}-${equipmentRng.int(10000, 99999)}`, String(equipmentRng.int(2009, 2024)), money(equipmentRng.amount(800000, 46000000, 100))]), { size: 8.5 });
  flow.heading('Schedule 5.11 - Finished Goods Inventory by Product', { level: 3 });
  const products = ['Strawberry Rhubarb Preserves', 'Blackberry Sage Jam', 'Peach Bourbon Preserves', 'Wild Blueberry Preserves', 'Fig and Black Pepper Jam', 'Sour Cherry Preserves', 'Apple Butter', 'Pumpkin Butter', 'Hot Pepper Jelly', 'Tomato Jam', 'Apricot Ginger Preserves', 'Raspberry Seedless Jam', 'Muscadine Jelly', 'Pear Cardamom Butter', 'Cherry Pie Filling (foodservice)', 'Apple Pie Filling (foodservice)', 'Lemon Curd', 'Orange Marmalade'];
  const inventoryRng = rng.fork('inventory');
  flow.table([{ header: 'SKU', width: 0.13 }, { header: 'Product', width: 0.4 }, { header: 'Pack', width: 0.17 }, { header: 'Cases on hand', width: 0.14, align: 'right' }, { header: 'Cost / case', width: 0.16, align: 'right' }], products.map((product, index) => [`BC-${String(1000 + index * 17)}`, product, product.includes('foodservice') ? '6 x #10 can' : inventoryRng.pick(['12 x 10 oz', '12 x 18 oz', '6 x 32 oz']), String(inventoryRng.int(140, 2600)), `$${amount(inventoryRng.int(1840, 5400))}`]), { size: 8.5 });
  flow.heading('Schedule 5.13 - Employee Benefit Plans', { level: 3 });
  flow.table([{ header: 'Plan', width: 0.4 }, { header: 'Provider', width: 0.3 }, { header: 'Notes', width: 0.3 }], [
    ['Group medical (PPO and high-deductible options)', 'Meadowlark Health Plan', 'Seller pays 80% of employee-only premium'],
    ['Group dental and vision', 'Brightline Dental', 'Employee-paid'],
    ['401(k) plan with safe harbor match', 'Saltmarsh Retirement Services', '100% of first 3%, 50% of next 2%'],
    ['Paid time off policy', 'Internal', '10 to 20 days by tenure'],
    ['Annual production bonus', 'Internal', 'Paid each December; $1,500 average in 2025'],
  ], { size: 8.5 });
  flow.heading('Schedule 5.12 - Employees', { level: 3 });
  const staff = people(rng.fork('staff'), 22, { exclude: members });
  const titles = ['Plant Manager', 'Production Supervisor', 'Kettle Operator', 'Kettle Operator', 'Filler Operator', 'Filler Operator', 'Line Lead', 'Quality Assurance Manager', 'QA Technician', 'Maintenance Technician', 'Maintenance Technician', 'Warehouse Lead', 'Warehouse Associate', 'Warehouse Associate', 'Delivery Driver', 'Purchasing Coordinator', 'Customer Service Lead', 'Sales Manager - Retail', 'Sales Manager - Foodservice', 'Accounting Manager', 'Retail Shop Supervisor', 'Product Developer'];
  const staffRng = rng.fork('pay');
  flow.table([{ header: 'Employee', width: 0.28 }, { header: 'Title', width: 0.3 }, { header: 'Hire date', width: 0.2 }, { header: 'Pay', width: 0.22, align: 'right' }], staff.map((name, index) => [name, titles[index], numericDate(addDays('2008-03-01', staffRng.int(0, 6200))), index < 7 || [9, 10, 12, 13, 14, 20].includes(index) ? `$${amount(staffRng.int(1750, 3400))}/hr` : `$${amount(staffRng.amount(5200000, 12800000, 50000)).slice(0, -3)}/yr`]), { size: 8.5 });
  flow.heading('Schedule 5.15 - Permits, Certifications, and Litigation', { level: 3 });
  flow.table([{ header: 'Permit or certification', width: 0.42 }, { header: 'Number', width: 0.24 }, { header: 'Expires', width: 0.34 }], [
    ['Food facility registration (federal)', '17402288561', 'Renewal due in the next biennial period'],
    ['State food processing license', 'NC-FP-30418', longDate('2026-12-31')],
    ['Acidified foods process filings (14 products)', 'SID 2019-07-112 through 2024-03-221', 'No expiry; refile on formula change'],
    ['Organic handler certification', 'OCP-77214', longDate('2027-01-31')],
    ['Kosher certification', 'KS-55120', longDate('2026-08-31')],
    ['Third-party food safety audit certificate', 'FSA-2025-9932', longDate('2026-11-30')],
    ['Air permit (boiler)', 'AQ-14-2207', longDate('2028-06-30')],
  ], { size: 8.5 });
  flow.paragraph('Litigation: Hollis v. Bramblecrest Preserves LLC, small claims action filed in Briarport District Court alleging a chipped tooth from a cherry pit; claim of $4,800 tendered to Seller\'s product liability insurer, which is defending under reservation of rights. No other pending or threatened actions.', { size: 8.5 });
  flow.heading('Schedule 5.17 - Customers and Suppliers (fiscal 2025)', { level: 3 });
  const customers = companies(rng.fork('customers'), 10, { exclude: [buyer, seller, ...contractCounterparties] });
  const suppliers = companies(rng.fork('suppliers'), 10, { exclude: [buyer, seller, ...contractCounterparties, ...customers] });
  const salesRng = rng.fork('sales');
  const customerSales = customers.map(() => salesRng.amount(32000000, 310000000, 100)).sort((a, b) => b - a);
  const supplierSpend = suppliers.map(() => salesRng.amount(12000000, 190000000, 100)).sort((a, b) => b - a);
  flow.table([{ header: 'Rank', width: 0.07, align: 'right' }, { header: 'Customer', width: 0.33 }, { header: 'Net sales', width: 0.12, align: 'right' }, { header: 'Supplier', width: 0.34 }, { header: 'Purchases', width: 0.14, align: 'right' }], customers.map((name, index) => [String(index + 1), name, money(customerSales[index], { cents: false }), suppliers[index], money(supplierSpend[index], { cents: false })]), { size: 8 });
  flow.heading('Schedule 5.18 - Insurance', { level: 3 });
  flow.table([{ header: 'Coverage', width: 0.3 }, { header: 'Insurer', width: 0.3 }, { header: 'Policy period', width: 0.24 }, { header: 'Limit', width: 0.16, align: 'right' }], [
    ['Commercial general liability', 'Highmeadow Casualty Company', `${numericDate('2025-07-01')} - ${numericDate('2026-07-01')}`, '$1,000,000'],
    ['Product liability and recall', 'Northfell Mutual Insurance Company', `${numericDate('2025-07-01')} - ${numericDate('2026-07-01')}`, '$5,000,000'],
    ['Property (plant and contents)', 'Highmeadow Casualty Company', `${numericDate('2025-07-01')} - ${numericDate('2026-07-01')}`, '$14,500,000'],
    ['Commercial auto', 'Saltash Point Indemnity Co.', `${numericDate('2025-10-15')} - ${numericDate('2026-10-15')}`, '$1,000,000'],
    ['Umbrella', 'Northfell Mutual Insurance Company', `${numericDate('2025-07-01')} - ${numericDate('2026-07-01')}`, '$10,000,000'],
  ], { size: 8.5 });
  flow.heading('Schedule 1.1 - Accounting Principles (illustrative Closing Working Capital)', { level: 3 });
  flow.table([{ header: 'Line item', width: 0.6 }, { header: `As of ${longDate(balanceSheet)}`, width: 0.4, align: 'right' }], [
    ['Inventory - finished goods', money(171800000)], ['Inventory - raw materials and packaging', money(114600000)], ['Trade receivables, net of $86,000 reserve', money(197250000)], ['Prepaid expenses', money(9400000)],
    ['Trade payables', `(${money(141200000)})`], ['Accrued expenses (excluding income taxes)', `(${money(33850000)})`], [{ text: 'Working capital', face: 'sans-bold' }, { text: money(318000000), face: 'sans-bold' }],
  ], { size: 8.5 });
  flow.paragraph('Inventory is valued at the lower of first-in, first-out cost and net realizable value. Finished goods within sixty days of their best-by date are reserved at 50% and those past it at 100%. Receivables more than ninety days past due are fully reserved unless collected before the working capital calculation is delivered.', { size: 8.5 });
  flow.heading('Schedule 1.2 - Permitted Encumbrances', { level: 3 });
  for (const item of [
    'UCC-1 financing statement no. 20210044172 filed by Ollerton Equipment Leasing for filler line 2, which is leased equipment and is released at Closing by payoff.',
    'Landlord\'s statutory lien under the Orchard Row lease for rent not yet due.',
    'Purchase-money security interest of Wrenfield Electric Utility in the backup generator under the 2023 installment agreement, to be paid off at Closing.',
    'Liens for 2026 personal property taxes on the Briarport plant equipment, not yet due and payable.',
  ]) flow.paragraph(`- ${item}`, { size: 10, indent: 10 });
  flow.heading('Schedule 5.16 - Environmental Matters', { level: 3 });
  flow.paragraph('Phase I environmental site assessment of 1410 Orchard Row prepared by Quarry Bend Environmental Consultants, report no. QBE-25-0612. Findings: one historical recognized environmental condition (a former heating oil tank removed in 2009 with a no-further-action letter from the state); no current recognized environmental conditions; wastewater pretreatment permit in good standing; fruit waste is composted off site by a licensed hauler. The plant\'s boiler is permitted under air permit AQ-14-2207.', { size: 10 });
  flow.heading('Schedule 5.19 - Affiliate Transactions', { level: 3 });
  flow.table([{ header: 'Affiliate', width: 0.28 }, { header: 'Transaction', width: 0.5 }, { header: 'Annual amount', width: 0.22, align: 'right' }], [
    ['Bramble Family Orchard (owned by the Members)', 'Supply of strawberries and rhubarb at market price under an annual purchase order', '$184,000'],
    [members[1], 'Lease of a refrigerated trailer to the Business, month to month', '$9,600'],
    [members[0], 'Personal guaranty of the Kingsfold Community Bank line of credit (released at Closing)', 'None'],
  ], { size: 8.5 });
  flow.heading('Schedule 5.20 - Consumer Complaints (foreign material, illness, injury)', { level: 3 });
  const complaintRng = rng.fork('complaints');
  const complaintTypes = ['Cherry pit in Sour Cherry Preserves', 'Glass shard reported, not substantiated on lab review', 'Stomach upset after Apple Butter, no product defect found', 'Metal staple in carton (outer packaging only)', 'Mold under lid, jar seal compromised in transit', 'Plastic fragment from gasket in Peach Bourbon Preserves', 'Allergic reaction reported; product correctly labelled', 'Chipped jar rim at retail'];
  flow.table([{ header: 'Received', width: 0.18 }, { header: 'Description', width: 0.52 }, { header: 'Resolution', width: 0.3 }], complaintTypes.map((description, index) => [numericDate(addDays('2023-04-10', index * 120 + complaintRng.int(0, 60))), description, complaintRng.pick(['Refund and replacement', 'Refund; supplier corrective action', 'Investigated; no action required', 'Referred to insurer; closed', 'Lot reviewed; no further reports'])]), { size: 8.5 });
  flow.pageBreak();
  flow.heading('Exhibit A - Form of Bill of Sale', { level: 2 });
  flow.paragraph(`For good and valuable consideration, ${seller} ("Seller") sells, assigns, transfers, and delivers to ${buyer} ("Buyer") all of Seller's right, title, and interest in the Purchased Assets, as defined in the Asset Purchase Agreement between Seller and Buyer (the "Purchase Agreement"), free and clear of all encumbrances other than Permitted Encumbrances. This Bill of Sale is delivered under the Purchase Agreement, does not expand or limit any representation, warranty, covenant, or remedy in it, and is governed by Delaware law. Seller appoints Buyer its attorney-in-fact to collect and enforce the Purchased Assets in Seller's name for Buyer's benefit.`, { size: 10 });
  flow.pageBreak();
  flow.heading('Exhibit B - Principal Terms of the Escrow Agreement', { level: 2 });
  flow.table([{ header: 'Term', width: 0.3 }, { header: 'Provision', width: 0.7 }], [
    ['Escrow agent', escrowAgent],
    ['Escrow amount', `${money(escrow)} deposited at Closing`],
    ['Investment', 'Insured demand deposit account; interest follows the escrow funds and is taxed to Buyer until release'],
    ['Claims', 'Buyer may deliver a claim notice at any time before release; Seller has 30 days to object; undisputed amounts paid within 5 business days'],
    ['Release', 'Half of the then-remaining balance, less pending claims, at 12 months; the remainder, less pending claims, at 18 months after Closing'],
    ['Fees', 'Acceptance fee of $3,500 and annual fee of $2,500, shared equally'],
  ], { size: 8.5 });
  flow.heading('Exhibit C - Transition Services', { level: 2 });
  flow.table([{ header: 'Service', width: 0.44 }, { header: 'Duration', width: 0.22 }, { header: 'Monthly fee', width: 0.34, align: 'right' }], [
    ['ERP system access and order entry support', 'Up to 6 months', '$12,500'],
    ['Payroll processing for Transferred Employees', 'Up to 3 months', '$2,100'],
    ['Customer invoicing and cash application', 'Up to 4 months', '$4,800'],
    ['EDI connections with retail customers', 'Until Buyer cut-over, max 6 months', '$1,650'],
    ['Recipe management system read access', '6 months', 'No charge'],
    ['Consulting by Calla Bramble (separate agreement)', '12 months', 'Per consulting agreement'],
  ], { size: 8.5 });
  flow.pageBreak();
  flow.heading('Exhibit D - Form of Trademark Assignment', { level: 2 });
  flow.paragraph(`WHEREAS ${seller} ("Assignor") owns the trademarks, registrations, and applications listed in Schedule 5.8 of the Asset Purchase Agreement (the "Marks"), and ${buyer} ("Assignee") is acquiring them under that agreement; NOW THEREFORE, for good and valuable consideration, Assignor assigns to Assignee all right, title, and interest in the Marks, together with the goodwill of the business symbolized by them, all rights to sue for past infringement, and all income and royalties due after the date of this assignment. Assignor will sign any further documents Assignee reasonably requests to record this assignment with the United States Patent and Trademark Office and any foreign registry.`, { size: 10 });
  flow.pageBreak();
  flow.heading('Exhibit E - Principal Terms of the Consulting Agreement', { level: 2 });
  flow.table([{ header: 'Term', width: 0.3 }, { header: 'Provision', width: 0.7 }], [
    ['Consultant', members[0]],
    ['Services', 'New product development, recipe transfer to Buyer\'s plants, and introductions to key customers and growers'],
    ['Time commitment', 'Up to 20 hours per week for months 1 to 6; up to 10 hours per week for months 7 to 12'],
    ['Fee', '$14,000 per month, plus pre-approved travel expenses'],
    ['Term', 'Twelve months from the Closing Date; Buyer may extend for six months on the same terms'],
    ['Restrictive covenants', 'Those in Section 7.5 of the Asset Purchase Agreement, plus assignment to Buyer of recipes developed during the term'],
  ], { size: 8.5 });
  const pages = flow.finish();
  if (pages.length !== 25) throw new Error(`${id}: expected 25 pages, laid out ${pages.length}`);
  const { bytes, text } = digitalPdf(pages);
  return result({
    id,
    extension: 'pdf',
    files: [{ name: `${id}.pdf`, bytes }],
    text,
    title: 'Twenty-five-page asset purchase agreement with disclosure schedules',
    kind: 'contract',
    textLayer: 'native',
    pages: pages.length,
    categories: ['pages_25', 'contract', 'competing_dates', 'referenced_agreement', 'irrelevant_names', 'table', 'information_dense'],
    notes: `"Dated as of ${longDate(dated)}" on the cover and again in the preamble on page 3. The closing date (${longDate(closing)}), the outside date (${longDate(outside)}), and the balance sheet date (${longDate(balanceSheet)}) recur in the articles, and the disclosure schedules list ${contracts.length} contracts, trademark registrations, leases, permits, and insurance policies, each with its own dates - dozens of referenced agreements. The members join only for the non-compete; employees, customers, suppliers, and the escrow agent are named but are not parties.`,
    gold: gold({
      type: 'Asset Purchase Agreement',
      date: dated,
      role: 'effective',
      forbiddenDates: [[closing, 'scheduled closing date'], [outside, 'outside date for termination'], [balanceSheet, 'balance sheet date'], ['2016-05-01', 'date of the plant lease in the schedules']],
      parties: [buyer, seller],
      relation: 'between',
      roles: [[buyer, 'buyer'], [seller, 'seller']],
      forbiddenParties: [[members[0], 'member joining only for the non-compete'], [escrowAgent, 'escrow agent'], [contractCounterparties[0], 'counterparty to an assigned contract'], [customers[0], 'largest customer']],
      facts: [[money(price), '14,250,000', '$14.25 million'], ['preserves', 'jams', 'fruit'], [seller, 'Bramblecrest']],
      subjectTerms: ['asset purchase', 'preserves', 'purchase price', 'earn-out'],
      readiness: 'ready',
      dateText: [longDate(dated)],
    }),
  });
}
