/// Commercial contract clauses, written out in full and parameterised.
///
/// Contracts in the corpus are long because real ones are: the distiller has
/// to find one effective date among pages of obligations. These are not
/// filler - each clause states its own specifics (notice periods, caps,
/// insurance limits, addresses) drawn per document from a seeded generator,
/// and a contract uses each clause once.
import { money, wordsAndDigits } from '../lib/format.mjs';

/// ctx: { rng, a, b, aName, bName, state, county, venueCity, aAddress, bAddress, aNotice, bNotice, product }
/// `a` is the provider/seller/licensor role word ("Provider"), `b` the
/// customer role word ("Customer").
export const COMMERCIAL = [
  ['Services and Statements of Work', (c) => [
    `${c.a} will perform the services described in each statement of work that the parties execute under this Agreement (each, a "Statement of Work") and deliver the deliverables identified in it (the "Deliverables"). Each Statement of Work will describe the scope of the services, the Deliverables, the assumptions on which pricing is based, the milestones and acceptance criteria, the fees, and the names of any key personnel.`,
    `Each Statement of Work is governed by this Agreement. If a Statement of Work conflicts with this Agreement, this Agreement controls unless the Statement of Work expressly identifies the section of this Agreement it overrides and states that it does so. Neither party is obligated to enter into any Statement of Work.`,
  ]],
  ['Change Orders', (c) => [
    `Either party may request a change to the scope of a Statement of Work by written notice. Within ${wordsAndDigits(c.rng.pick([5, 7, 10]))} business days after receiving a change request, ${c.a} will provide a written estimate of the effect of the change on the fees, the schedule, and the Deliverables. No change is binding until both parties sign a change order describing it (a "Change Order").`,
    `${c.a} will continue to perform the services as originally described while a change request is pending, unless ${c.b} directs otherwise in writing. Time spent preparing estimates for changes requested by ${c.b} that exceed ${wordsAndDigits(c.rng.pick([4, 6, 8]))} hours in any calendar month is billable at the rates in the applicable Statement of Work.`,
  ]],
  ['Personnel and Subcontractors', (c) => [
    `${c.a} will assign personnel with the training and experience the services require. ${c.a} will not remove or replace any key personnel named in a Statement of Work during the first ${wordsAndDigits(c.rng.pick([90, 120, 180]))} days of their assignment except for illness, resignation, termination of employment, or a leave of absence, and will give ${c.b} at least ${wordsAndDigits(c.rng.pick([10, 15]))} days' notice of any other replacement.`,
    `${c.a} may use subcontractors only with ${c.b}'s prior written consent, which ${c.b} will not unreasonably withhold. ${c.a} remains responsible for the performance of its subcontractors and for their compliance with this Agreement as if their acts and omissions were its own.`,
    `While on ${c.b}'s premises, ${c.a} personnel will comply with ${c.b}'s site security, safety, and conduct policies made available to ${c.a} in advance. ${c.b} may require the removal of any individual who violates those policies.`,
  ]],
  ['Fees and Payment', (c) => {
    const days = c.rng.pick([30, 45]);
    const late = c.rng.pick(['one percent (1%)', 'one and one-half percent (1.5%)']);
    return [
      `${c.b} will pay the fees stated in each Statement of Work. Unless a Statement of Work provides otherwise, ${c.a} will invoice time-and-materials fees monthly in arrears and fixed fees on completion of the milestone to which they relate. Each invoice will itemize the services performed, the hours worked by each individual for time-and-materials work, and any reimbursable expenses, with receipts for any single expense above ${money(c.rng.pick([7500, 10000, 25000]), { cents: false })}.`,
      `${c.b} will pay each undisputed invoice within ${wordsAndDigits(days)} days after receipt. Undisputed amounts not paid when due bear interest at ${late} per month or the maximum rate permitted by law, whichever is less, from the due date until paid.`,
      `If ${c.b} disputes any portion of an invoice in good faith, it will notify ${c.a} in writing before the payment due date, describing the reason for the dispute, and will pay the undisputed portion. The parties will work together to resolve the dispute within ${wordsAndDigits(c.rng.pick([30, 45]))} days. ${c.a} will not suspend the services because of an amount disputed in good faith.`,
    ];
  }],
  ['Taxes', (c) => [
    `Fees are exclusive of sales, use, value-added, and similar taxes. ${c.b} will pay any such taxes that ${c.a} is required to collect, as separately stated on ${c.a}'s invoices, unless ${c.b} provides a valid exemption certificate. Each party is responsible for taxes on its own income, property, and employees.`,
    `If ${c.b} is required by law to withhold any tax from a payment, it will pay the withheld amount to the appropriate authority, provide ${c.a} with an official receipt, and cooperate with ${c.a} in claiming any available credit or refund.`,
  ]],
  ['Expenses', (c) => [
    `${c.b} will reimburse ${c.a} for reasonable travel and out-of-pocket expenses incurred in performing the services, if approved in advance by ${c.b} and incurred in accordance with ${c.b}'s travel policy then in effect. Air travel will be booked in economy class for flights under ${wordsAndDigits(c.rng.pick([4, 5, 6]))} hours. Expenses are billed at cost without markup.`,
  ]],
  ['Confidentiality', (c) => [
    `"Confidential Information" means all non-public information disclosed by or on behalf of a party (the "Discloser") to the other party (the "Recipient") that is marked confidential or that a reasonable person would understand to be confidential, including business plans, pricing, customer information, technical information, and the terms of this Agreement. Confidential Information does not include information that is or becomes public through no fault of the Recipient, that the Recipient already knew without restriction, that the Recipient independently develops, or that a third party lawfully discloses to the Recipient without restriction.`,
    `The Recipient will use the Discloser's Confidential Information only to perform its obligations or exercise its rights under this Agreement, will protect it with at least the care it uses to protect its own information of similar sensitivity and no less than reasonable care, and will disclose it only to its employees, contractors, and advisers who need to know it and are bound by obligations of confidentiality at least as protective as these.`,
    `The Recipient may disclose Confidential Information when required by law or court order if it gives the Discloser prompt notice (where legally permitted) and reasonable assistance in seeking a protective order. These obligations continue for ${wordsAndDigits(c.rng.pick([3, 5]))} years after this Agreement ends, and for trade secrets for as long as they remain trade secrets.`,
  ]],
  ['Data Protection and Security', (c) => [
    `If ${c.a} processes personal information on behalf of ${c.b}, it will do so only on ${c.b}'s documented instructions and in accordance with the data processing terms attached as an exhibit to the applicable Statement of Work. ${c.a} will maintain administrative, physical, and technical safeguards appropriate to the nature of the information, including encryption of personal information in transit and at rest, multi-factor authentication for remote access, and logging of administrative access.`,
    `${c.a} will notify ${c.b} without undue delay, and in any event within ${wordsAndDigits(c.rng.pick([24, 48, 72]))} hours, after becoming aware of any unauthorized access to or disclosure of ${c.b}'s data in ${c.a}'s possession or control, and will cooperate with ${c.b}'s investigation and any notifications ${c.b} is required to make.`,
  ]],
  ['Intellectual Property', (c) => [
    `Each party retains all right, title, and interest in the intellectual property it owned before the Effective Date or develops independently of this Agreement ("Pre-Existing IP"). Subject to payment of the applicable fees, ${c.a} assigns to ${c.b} all right, title, and interest in the Deliverables, excluding any ${c.a} Pre-Existing IP incorporated in them.`,
    `${c.a} grants ${c.b} a non-exclusive, perpetual, royalty-free license to use, copy, and modify any ${c.a} Pre-Existing IP incorporated in a Deliverable, solely as part of that Deliverable and for ${c.b}'s internal business purposes. ${c.a} may use the general skills, know-how, and experience its personnel acquire in performing the services, provided it does not disclose ${c.b}'s Confidential Information.`,
  ]],
  ['Warranties', (c) => [
    `${c.a} warrants that the services will be performed in a professional and workmanlike manner consistent with generally accepted industry standards, and that each Deliverable will conform in all material respects to its specifications for ${wordsAndDigits(c.rng.pick([30, 60, 90]))} days after acceptance. ${c.b}'s exclusive remedy, and ${c.a}'s sole obligation, for breach of this warranty is for ${c.a} to re-perform the nonconforming services or correct the Deliverable at no additional charge or, if ${c.a} cannot do so within a reasonable time, to refund the fees paid for the nonconforming portion.`,
    'Each party warrants that it has full power and authority to enter into this Agreement, that its execution has been duly authorized, and that its performance will not breach any other agreement by which it is bound.',
    'EXCEPT AS EXPRESSLY PROVIDED IN THIS AGREEMENT, NEITHER PARTY MAKES ANY OTHER WARRANTY, EXPRESS OR IMPLIED, INCLUDING ANY IMPLIED WARRANTY OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, OR NON-INFRINGEMENT.',
  ]],
  ['Indemnification', (c) => [
    `${c.a} will defend ${c.b} and its officers, directors, and employees against any claim by a third party alleging that a Deliverable, as delivered, infringes or misappropriates that third party's intellectual property rights, and will pay the damages, costs, and reasonable attorneys' fees finally awarded or agreed in settlement. This obligation does not apply to a claim arising from a modification not made by ${c.a}, a combination with materials not supplied by ${c.a}, or ${c.b}'s specifications.`,
    `Each party will defend the other against third-party claims for bodily injury, death, or damage to tangible property caused by the negligence or willful misconduct of the indemnifying party or its personnel while performing under this Agreement, and will pay the resulting damages and costs.`,
    'The indemnified party must give prompt written notice of the claim, allow the indemnifying party to control its defense and settlement, and provide reasonable cooperation at the indemnifying party\'s expense. The indemnifying party may not settle a claim in a way that imposes any obligation on the indemnified party without its prior written consent.',
  ]],
  ['Limitation of Liability', (c) => {
    const months = c.rng.pick([12, 18, 24]);
    return [
      `EXCEPT FOR A PARTY'S INDEMNIFICATION OBLIGATIONS, BREACH OF ITS CONFIDENTIALITY OBLIGATIONS, OR GROSS NEGLIGENCE OR WILLFUL MISCONDUCT, NEITHER PARTY IS LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES, OR FOR LOST PROFITS OR REVENUE, HOWEVER CAUSED.`,
      `EACH PARTY'S TOTAL LIABILITY ARISING OUT OF OR RELATING TO THIS AGREEMENT WILL NOT EXCEED THE FEES PAID AND PAYABLE TO ${c.a.toUpperCase()} UNDER THIS AGREEMENT DURING THE ${wordsAndDigits(months).toUpperCase()} MONTHS BEFORE THE EVENT GIVING RISE TO THE CLAIM. THIS LIMIT DOES NOT APPLY TO ${c.b.toUpperCase()}'S OBLIGATION TO PAY FEES WHEN DUE.`,
    ];
  }],
  ['Insurance', (c) => {
    const general = c.rng.pick([100000000, 200000000]);
    const professional = c.rng.pick([200000000, 300000000, 500000000]);
    const cyber = c.rng.pick([300000000, 500000000]);
    return [
      `During the term and for one year afterward, ${c.a} will maintain, with insurers rated A- VII or better by a recognized rating agency: (a) commercial general liability insurance of at least ${money(general, { cents: false })} per occurrence and ${money(general * 2, { cents: false })} in the aggregate; (b) professional liability (errors and omissions) insurance of at least ${money(professional, { cents: false })} per claim; (c) cyber liability insurance of at least ${money(cyber, { cents: false })} per claim, covering data breach response costs; (d) workers' compensation insurance as required by law; and (e) employer's liability insurance of at least ${money(100000000, { cents: false })}.`,
      `${c.a} will name ${c.b} as an additional insured on the commercial general liability policy and will deliver certificates of insurance on request. ${c.a} will give ${c.b} at least ${wordsAndDigits(30)} days' prior written notice of cancellation or material reduction of any required coverage.`,
    ];
  }],
  ['Term and Termination', (c) => {
    const years = c.rng.pick([2, 3, 5]);
    return [
      `This Agreement begins on the Effective Date and continues for an initial term of ${wordsAndDigits(years)} years. It then renews automatically for successive one-year renewal terms unless either party gives notice of non-renewal at least ${wordsAndDigits(c.rng.pick([60, 90]))} days before the end of the then-current term. Each Statement of Work remains in effect for the term stated in it, and this Agreement continues to govern any Statement of Work that is in effect when this Agreement ends.`,
      `${c.b} may terminate this Agreement or any Statement of Work for convenience on ${wordsAndDigits(c.rng.pick([30, 45, 60]))} days' written notice. Either party may terminate this Agreement or an affected Statement of Work if the other party materially breaches it and fails to cure the breach within ${wordsAndDigits(30)} days after receiving written notice describing it, or immediately if the other party becomes insolvent, makes an assignment for the benefit of creditors, or becomes subject to bankruptcy proceedings that are not dismissed within ${wordsAndDigits(60)} days.`,
      `On termination, ${c.b} will pay for services performed and expenses incurred through the effective date of termination, and ${c.a} will deliver all completed and in-progress Deliverables and return or destroy ${c.b}'s Confidential Information. Sections addressing confidentiality, intellectual property, indemnification, limitation of liability, and payment obligations survive termination.`,
    ];
  }],
  ['Non-Solicitation', (c) => [
    `During the term and for ${wordsAndDigits(c.rng.pick([6, 12]))} months afterward, neither party will solicit for employment any employee of the other party who was directly involved in the services, without the other party's prior written consent. General solicitations not targeted at such employees, and hiring anyone who responds to them, do not violate this section.`,
  ]],
  ['Records and Audit', (c) => [
    `${c.a} will keep complete and accurate records of time worked and expenses billed under this Agreement for at least ${wordsAndDigits(c.rng.pick([3, 4]))} years after the related invoice. On ${wordsAndDigits(c.rng.pick([15, 30]))} days' notice and not more than once in any twelve-month period, ${c.b} or an independent auditor bound by confidentiality obligations may audit those records during normal business hours. If an audit reveals an overcharge, ${c.a} will promptly refund it, and will also bear the reasonable cost of the audit if the overcharge exceeds five percent (5%) of the amounts billed for the audited period.`,
  ]],
  ['Compliance with Laws', (c) => [
    `Each party will comply with all laws and regulations applicable to its performance under this Agreement, including anti-bribery and anti-corruption laws, export control and economic sanctions laws, and employment laws. Neither party will offer or give anything of value to any government official or employee of the other party to obtain or retain business or any improper advantage.`,
  ]],
  ['Force Majeure', (c) => [
    `Neither party is liable for a delay or failure to perform caused by events beyond its reasonable control, including natural disasters, epidemics, war, terrorism, labor disputes not involving its own employees, and failures of public utilities or communications networks, provided it notifies the other party promptly and uses reasonable efforts to resume performance. If such an event prevents performance of a material obligation for more than ${wordsAndDigits(c.rng.pick([30, 45, 60]))} consecutive days, either party may terminate the affected Statement of Work on written notice. This section does not excuse ${c.b}'s obligation to pay for services performed.`,
  ]],
  ['Assignment', (c) => [
    `Neither party may assign this Agreement without the other party's prior written consent, except that either party may assign it without consent to an affiliate or to a successor in a merger, acquisition, or sale of all or substantially all of the assets of the business to which this Agreement relates, if the assignee agrees in writing to be bound by it. Any other attempted assignment is void. This Agreement binds and benefits the parties and their permitted successors and assigns.`,
  ]],
  ['Notices', (c) => [
    `Notices under this Agreement must be in writing and delivered by hand, by nationally recognized overnight courier, or by certified mail, return receipt requested, to the address below (or another address a party designates by notice). A notice is effective on delivery. Routine operational communications may be sent by email.`,
    `If to ${c.a}: ${c.aName}, ${c.aAddress}, Attention: ${c.aNotice}. If to ${c.b}: ${c.bName}, ${c.bAddress}, Attention: ${c.bNotice}.`,
  ]],
  ['Governing Law and Disputes', (c) => [
    `This Agreement is governed by the laws of the State of ${c.state}, without regard to its conflict of laws rules. The parties will first attempt to resolve any dispute through good-faith negotiation between executives with authority to settle it, beginning within ${wordsAndDigits(10)} business days after either party's written request. If the dispute is not resolved within ${wordsAndDigits(30)} days after that request, either party may bring proceedings exclusively in the state or federal courts located in ${c.venue}, and each party consents to the jurisdiction of those courts. Either party may seek injunctive relief in any court of competent jurisdiction to protect its Confidential Information or intellectual property.`,
  ]],
  ['Independent Contractors', (c) => [
    `The parties are independent contractors. Nothing in this Agreement creates a partnership, joint venture, agency, franchise, or employment relationship. ${c.a} is solely responsible for the compensation, benefits, and supervision of its personnel and for withholding and paying all employment taxes for them.`,
  ]],
  ['Publicity', (c) => [
    `Neither party will use the other party's name, logo, or trademarks in any press release, customer list, or marketing material without the other party's prior written consent, which may be withdrawn at any time on notice.`,
  ]],
  ['General', (c) => [
    `This Agreement, together with its exhibits and each Statement of Work, is the entire agreement of the parties on its subject matter and supersedes all prior and contemporaneous agreements, proposals, and understandings, whether written or oral. Pre-printed terms on any purchase order, invoice, or acknowledgement have no effect. This Agreement may be amended only by a written instrument signed by authorized representatives of both parties.`,
    `No waiver is effective unless in writing, and no failure or delay in exercising a right operates as a waiver of it. If any provision is held unenforceable, it will be enforced to the maximum extent permissible and the remaining provisions will remain in effect. This Agreement may be executed in counterparts, including by electronic signature, each of which is an original and all of which together are one instrument.`,
  ]],
];

/// Lays out clauses as numbered sections. A one-paragraph clause runs in
/// after its bold heading ("7. Taxes. Fees are..."); a longer one gets a
/// heading line and numbered subsections ("7.1", "7.2"). Returns the next
/// section number.
export function writeClauses(flow, clauses, ctx, { start = 1, size, headingFace = 'sans-bold', upper = false } = {}) {
  let number = start;
  for (const [title, body] of clauses) {
    const paragraphs = body(ctx);
    const heading = upper ? title.toUpperCase() : title;
    if (paragraphs.length === 1) {
      flow.paragraph([{ text: `${number}. ${heading}. `, face: headingFace }, { text: paragraphs[0] }], { size });
    } else {
      flow.paragraph([{ text: `${number}. ${heading}.`, face: headingFace }], { size, after: 2, keepWithNext: 30 });
      paragraphs.forEach((text, index) => flow.paragraph([{ text: `${number}.${index + 1} `, face: headingFace }, { text }], { size }));
    }
    number += 1;
  }
  return number;
}
