/// An office lease written out article by article, for the long scanned
/// lease. Each article states its own figures, so the twenty-five pages are
/// twenty-five pages of different obligations rather than one clause
/// repeated.
import { money, wordsAndDigits } from '../lib/format.mjs';

/// ctx: { landlord, tenant, building, suite, area, share, start, end, rent }
export function officeLeaseArticles(c) {
  return [
    ['Premises', [
      `Landlord leases to Tenant, and Tenant leases from Landlord, ${c.suite} on the fourth floor of the building known as ${c.building} (the "Building"), containing approximately ${c.area} rentable square feet as shown on Exhibit A (the "Premises"), together with the right to use, in common with other tenants, the lobbies, elevators, stairways, restrooms, loading dock, and parking areas of the Building (the "Common Areas").`,
      'The rentable area of the Premises has been measured under the standard method for measuring office floor area in effect on the date of this Lease and is not subject to remeasurement during the Term. Tenant has inspected the Premises and accepts them in their present condition, subject only to completion of the Landlord Work described in the Work Letter attached as Exhibit C.',
    ]],
    ['Term', [
      `The term of this Lease (the "Term") begins on ${c.start} (the "Commencement Date") and ends at midnight on ${c.end} (the "Expiration Date"), unless sooner terminated as provided in this Lease.`,
      'If Landlord cannot deliver possession of the Premises on the Commencement Date because the Landlord Work is not substantially complete, this Lease remains in effect, Landlord is not liable for the delay, and the Commencement Date and the Expiration Date will each be postponed by the number of days of delay. If possession has not been delivered within ninety days after the scheduled Commencement Date, Tenant may terminate this Lease by written notice given before possession is delivered.',
    ]],
    ['Base Rent', [
      `Tenant will pay base rent ("Base Rent") in the amounts set out in the rent schedule below, in equal monthly installments in advance on the first day of each calendar month during the Term, without demand, deduction, or setoff. Base Rent for the first full month is due on signing. Base Rent for any partial month is prorated on a daily basis.`,
      `Rent payments will be made by electronic funds transfer to the account Landlord designates in writing. Any installment of rent not received within five days after it is due bears a late charge equal to five percent of the installment, and any amount unpaid for thirty days bears interest at ${wordsAndDigits(12)} percent per year until paid.`,
    ]],
    ['Operating Expenses and Taxes', [
      `Beginning with the second calendar year of the Term, Tenant will pay, as additional rent, Tenant's Share (${c.share}) of the amount by which Operating Expenses and Taxes for each calendar year exceed Operating Expenses and Taxes for the base year, calendar year 2026 (the "Base Year"). Operating Expenses include the costs of operating, cleaning, insuring, repairing, and managing the Building and the Common Areas, but exclude capital improvements (other than those that reduce Operating Expenses, amortized over their useful lives), leasing commissions, Landlord's financing costs, and costs reimbursed by insurance.`,
      'Landlord will deliver an estimate of Tenant\'s share of excess Operating Expenses and Taxes before each calendar year, and Tenant will pay one-twelfth of the estimate monthly with Base Rent. Within one hundred twenty days after each year Landlord will deliver a statement of actual costs, and the parties will settle any difference within thirty days. Tenant may inspect Landlord\'s records supporting the statement within ninety days after receiving it.',
      'Controllable Operating Expenses, meaning all Operating Expenses other than taxes, insurance, utilities, and snow removal, may not increase by more than five percent per year on a cumulative, compounding basis over the Base Year amount.',
    ]],
    ['Security Deposit', [
      `On signing this Lease Tenant will deposit with Landlord ${money(2425000)} as security for the performance of its obligations (the "Security Deposit"). Landlord may apply the Security Deposit to cure any default by Tenant, and Tenant will restore the amount applied within ten days after demand. Landlord will return any unapplied balance within thirty days after the end of the Term and Tenant's surrender of the Premises.`,
    ]],
    ['Use', [
      'Tenant will use the Premises only for general office purposes, including the operation of a software development and customer support business, and for no other purpose. Tenant will not use the Premises in a manner that creates a nuisance, interferes with other tenants, overloads the floors or electrical systems, or violates any law or the certificate of occupancy for the Building.',
      'Tenant will comply with all laws applicable to its particular use of the Premises, including accessibility laws as they apply to Tenant\'s alterations, and Landlord will comply with laws applicable to the Common Areas and the structural elements of the Building.',
    ]],
    ['Services and Utilities', [
      'Landlord will furnish heating, ventilation, and air conditioning to the Premises from 7:00 a.m. to 7:00 p.m. on weekdays and 8:00 a.m. to 1:00 p.m. on Saturdays, excluding holidays; electricity for lighting and normal office equipment; passenger elevator service at all times; janitorial service five nights a week to the standard described in Exhibit D; and water for restrooms and the kitchenette.',
      'Heating or cooling outside those hours is available on request through the Building\'s online system at $65.00 per hour per floor zone, subject to adjustment for changes in utility rates. Tenant may install a supplemental cooling unit for its server room, at its cost, metered separately.',
      'Landlord is not liable for any interruption of services caused by repairs, accidents, or events beyond its control, but if an interruption within Landlord\'s reasonable control prevents Tenant from using the Premises for more than five consecutive business days, Base Rent abates from the sixth day until the service is restored.',
    ]],
    ['Repairs and Maintenance', [
      'Landlord will keep in good repair the roof, foundation, structural elements, exterior walls, Common Areas, and the base building plumbing, electrical, and mechanical systems serving the Premises. Tenant will keep the interior of the Premises in good condition and repair, including its own fixtures, supplemental equipment, and alterations, reasonable wear and tear and casualty excepted.',
      'Tenant will promptly notify Landlord of any damage or defect in the Premises that is Landlord\'s responsibility to repair. If Tenant fails to make a repair for which it is responsible within thirty days after notice, Landlord may make the repair and Tenant will reimburse its reasonable cost plus an administrative fee of ten percent.',
    ]],
    ['Alterations', [
      'Tenant may not make any alterations to the Premises without Landlord\'s prior written consent, except cosmetic alterations such as painting and carpeting that cost less than $25,000 in any year, do not require a building permit, and do not affect the structure or building systems. Landlord will not unreasonably withhold consent to other alterations.',
      'All alterations will be performed by licensed contractors approved by Landlord, in a good and workmanlike manner, in compliance with law and the Building\'s construction rules, and at Tenant\'s cost. At the end of the Term Tenant will remove any cabling it installed and any alterations Landlord designated for removal when it consented, and repair any damage caused by the removal.',
    ]],
    ['Assignment and Subletting', [
      'Tenant may not assign this Lease or sublet all or any part of the Premises without Landlord\'s prior written consent, which Landlord will not unreasonably withhold, condition, or delay. Tenant will pay Landlord fifty percent of any rent received under a sublease in excess of the rent payable under this Lease for the space sublet, after deducting Tenant\'s reasonable costs of subletting.',
      'Tenant may assign this Lease without consent to an entity that controls, is controlled by, or is under common control with Tenant, or to a successor by merger or by purchase of substantially all of Tenant\'s assets, if the successor has a net worth at least equal to Tenant\'s on the date of this Lease and assumes this Lease in writing. No assignment releases Tenant from its obligations.',
    ]],
    ['Insurance', [
      'Tenant will maintain throughout the Term commercial general liability insurance with limits of at least $1,000,000 per occurrence and $2,000,000 in the aggregate, umbrella liability insurance of at least $5,000,000, property insurance covering Tenant\'s personal property and alterations at full replacement cost, business interruption insurance covering at least six months of rent, and workers\' compensation insurance as required by law.',
      'Tenant\'s liability policies will name Landlord, its managing agent, and its mortgagee as additional insureds. Tenant will deliver certificates of insurance before taking possession and on each renewal. Landlord will maintain property insurance on the Building at full replacement cost and commercial general liability insurance on the Common Areas.',
      'Each party waives, and will cause its property insurers to waive, any right of recovery against the other for loss or damage to its property to the extent the loss is or should have been covered by the property insurance required by this Lease.',
    ]],
    ['Indemnification', [
      'Tenant will indemnify, defend, and hold harmless Landlord and its agents from claims for injury to persons or damage to property occurring in the Premises, or caused by the negligence or misconduct of Tenant or its employees, contractors, or invitees, except to the extent caused by Landlord\'s negligence or misconduct. Landlord will indemnify Tenant from claims arising in the Common Areas or caused by the negligence or misconduct of Landlord or its agents, except to the extent caused by Tenant.',
    ]],
    ['Casualty', [
      'If the Premises or the Building are damaged by fire or other casualty, Landlord will, within sixty days after the casualty, notify Tenant of its reasonable estimate of the time required to restore them. If the estimate exceeds two hundred seventy days, either party may terminate this Lease by notice given within thirty days after receiving the estimate.',
      'If this Lease is not terminated, Landlord will restore the base building and the Landlord Work with reasonable diligence, and Base Rent and Tenant\'s Share of Operating Expenses will abate in proportion to the part of the Premises that is unusable until it is restored. If the casualty occurs during the last twelve months of the Term, either party may terminate this Lease.',
    ]],
    ['Condemnation', [
      'If all of the Premises, or so much of them that Tenant cannot reasonably operate its business in the remainder, are taken by eminent domain, this Lease terminates on the date of the taking. If only part of the Premises is taken, Base Rent will be reduced in proportion to the area taken. All compensation for the taking belongs to Landlord, except any award made separately to Tenant for its moving expenses and trade fixtures.',
    ]],
    ['Default and Remedies', [
      'Each of the following is an event of default by Tenant: failure to pay any rent within five days after notice that it is past due; failure to perform any other obligation within thirty days after notice, or such longer period as is reasonably necessary if the failure cannot be cured within thirty days and Tenant diligently pursues the cure; abandonment of the Premises; and Tenant\'s bankruptcy or insolvency.',
      'If an event of default occurs, Landlord may terminate this Lease or terminate Tenant\'s right to possess the Premises without terminating this Lease, re-enter the Premises as allowed by law, and recover from Tenant all unpaid rent, the cost of reletting (including reasonable brokerage commissions and alterations), and the amount by which the rent for the remainder of the Term exceeds the fair rental value of the Premises, discounted to present value at the discount rate of the Federal Reserve Bank of San Francisco plus one percent. Landlord will use reasonable efforts to mitigate its damages.',
    ]],
    ['Landlord Default', [
      'Landlord is in default if it fails to perform any of its obligations within thirty days after written notice from Tenant, or such longer period as is reasonably necessary if Landlord diligently pursues the cure. Tenant\'s remedies are limited to actual damages and injunctive relief, and Landlord\'s liability is limited to its interest in the Building, including rents and insurance and sale proceeds.',
    ]],
    ['Subordination and Estoppel', [
      'This Lease is subordinate to any mortgage now or later encumbering the Building, provided the holder of the mortgage agrees in writing that Tenant\'s possession will not be disturbed so long as Tenant is not in default. Within ten business days after request, each party will deliver to the other a certificate stating whether this Lease is in full force, the dates through which rent has been paid, and whether any default exists.',
    ]],
    ['Surrender and Holding Over', [
      'At the end of the Term Tenant will surrender the Premises broom clean and in the condition required by this Lease, remove its personal property and any alterations it is required to remove, and return all keys and access cards. Any property left behind more than ten days after the end of the Term is deemed abandoned.',
      'If Tenant remains in possession after the end of the Term without Landlord\'s written consent, it is a tenant at sufferance and will pay Base Rent at 150% of the rate in effect at the end of the Term for the first sixty days and 200% thereafter, plus additional rent, and will be liable for Landlord\'s damages if the holdover continues more than thirty days after Landlord notifies Tenant that it has leased the Premises to another tenant.',
    ]],
    ['Parking', [
      'During the Term Tenant may use up to forty-two unreserved parking spaces in the Building\'s garage, of which up to four may be converted to reserved spaces, at the monthly rates Landlord charges from time to time for comparable spaces, initially $95.00 per unreserved space and $160.00 per reserved space. Tenant may also use the Building\'s secure bicycle room and the four electric vehicle charging stations on a first-come basis.',
    ]],
    ['Signs', [
      'Landlord will list Tenant\'s name in the Building directory and on the fourth-floor elevator lobby sign at Landlord\'s cost. Tenant may not place any sign visible from outside the Premises without Landlord\'s consent. So long as Tenant leases at least the entire Premises and is not in default, Tenant may install one panel on the monument sign at the Building entrance, at its cost and in conformity with the Building\'s sign criteria.',
    ]],
    ['Renewal Option', [
      'Tenant has one option to extend the Term for five years, exercisable by written notice given not less than twelve and not more than fifteen months before the Expiration Date, provided Tenant is not then in default and is occupying the entire Premises. Base Rent for the extension term will be ninety-five percent of the fair market rent for comparable space in comparable buildings, determined by agreement or, failing agreement within thirty days, by the appraisal procedure in Exhibit E.',
    ]],
    ['Right of First Offer', [
      'If any space on the fifth floor of the Building becomes available for lease during the first five years of the Term, Landlord will first offer it to Tenant on the terms on which Landlord intends to offer it to the market. Tenant may accept the offer by written notice within ten business days, failing which Landlord may lease the space to anyone on terms not materially more favorable to the tenant than those offered to Tenant.',
    ]],
    ['Hazardous Materials', [
      'Tenant will not bring onto the Premises any hazardous materials other than ordinary office and cleaning supplies in quantities customary for general office use, stored and used in compliance with law. Landlord represents that, to its knowledge, the Premises contain no asbestos-containing materials and no hazardous materials in violation of law on the date of this Lease.',
    ]],
    ['Access by Landlord', [
      'Landlord may enter the Premises on twenty-four hours\' notice (or without notice in an emergency) to inspect them, to make repairs and improvements, to show them to prospective purchasers and lenders, and, during the last twelve months of the Term, to prospective tenants. Landlord will use reasonable efforts to minimize interference with Tenant\'s business and will comply with Tenant\'s reasonable security procedures for its secure areas.',
    ]],
    ['Brokers', [
      'Each party represents that it has dealt with no broker in connection with this Lease other than Kingsfold Commercial Realty for Landlord and Oriel Tenant Advisors LLC for Tenant, whose commissions Landlord will pay under separate agreements, and each party will indemnify the other against claims of any other broker claiming through it.',
    ]],
    ['Notices', [
      `Notices under this Lease must be in writing and delivered by hand, by nationally recognized overnight courier, or by certified mail to the addresses in the Basic Lease Information, and are effective on delivery or refusal. Notices to Tenant after the Commencement Date may be delivered to the Premises.`,
    ]],
    ['Miscellaneous', [
      'This Lease, including its exhibits, is the entire agreement of the parties concerning the Premises and may be amended only in a writing signed by both parties. It is governed by the law of the state in which the Building is located. If any provision is held invalid, the remainder of this Lease is not affected. Time is of the essence of each provision. This Lease binds and benefits the parties and their permitted successors and assigns.',
      'Neither party will record this Lease, but either may record a short memorandum of it in a form reasonably acceptable to both. Submission of this Lease for examination is not an offer, and this Lease is effective only when signed by both parties and delivered.',
    ]],
  ];
}

/// Building rules, attached to the lease as Exhibit B.
export const BUILDING_RULES = [
  'Sidewalks, entrances, halls, elevators, and stairways may not be obstructed or used for any purpose other than entering and leaving the Premises.',
  'No sign, advertisement, or notice may be painted or affixed on any part of the Building visible from outside the Premises without Landlord\'s prior written consent.',
  'Deliveries of furniture, equipment, and supplies requiring the freight elevator must be scheduled with the Building office at least two business days in advance and made before 8:00 a.m. or after 6:00 p.m.',
  'Tenant may not install additional locks or change existing locks without Landlord\'s consent, and must deliver a key to any permitted lock to the Building office.',
  'Restrooms and plumbing fixtures may be used only for their intended purposes; Tenant will pay for repairing damage resulting from misuse by its employees or visitors.',
  'No cooking may be done in the Premises other than in microwave ovens and coffee makers in the kitchenette; no space heaters may be used.',
  'Tenant will not keep animals in the Premises other than service animals.',
  'Bicycles must be kept in the bicycle room and may not be brought into the Premises or the passenger elevators.',
  'Smoking and vaping are prohibited in the Building, the garage, and within twenty-five feet of any entrance.',
  'Tenant will close windows and blinds and turn off lights, coffee makers, and other appliances at the end of each business day.',
  'Access to the Building after 7:00 p.m. on weekdays and at all times on weekends and holidays requires an access card; Tenant is responsible for cards issued to its employees and will report lost cards promptly.',
  'Tenant will not overload floors; safes, file systems, and equipment weighing more than fifty pounds per square foot require Landlord\'s approval of their location.',
  'Waste must be separated for recycling and composting as the Building\'s program requires and placed only in the containers provided.',
  'Canvassing, soliciting, and peddling in the Building are prohibited.',
  'Landlord may close the Building in an emergency and may refuse admission to anyone who does not present identification during after-hours access periods.',
  'Tenant will comply with the Building\'s emergency procedures and will designate a floor warden to participate in the semi-annual fire drills.',
];

/// Further exhibits for the scanned lease, each on its own page in the
/// delivered document: what operating expenses exclude, the after-hours
/// service rates, parking rules, the form of commencement memorandum, the
/// form of estoppel certificate, and the sustainability provisions.
export function leaseExhibits(c) {
  return [
    ['EXHIBIT F - OPERATING EXPENSE EXCLUSIONS', [
      'Operating Expenses do not include any of the following:',
      '(a) costs of capital improvements, except those that reduce Operating Expenses (amortized over their useful life, but only up to the savings actually achieved in each year) or that are required by laws first taking effect after the Commencement Date;',
      '(b) depreciation, interest, principal, points, and fees on any debt secured by the Building, and rent under any ground lease;',
      '(c) leasing commissions, attorneys\' fees, advertising, and other costs of leasing space or of negotiating or enforcing leases with other tenants;',
      '(d) costs of improving or decorating space for other tenants, and allowances or concessions given to them;',
      '(e) costs of services provided to other tenants but not to Tenant, or provided to other tenants at a level greater than that provided to Tenant;',
      '(f) costs reimbursed by insurance, warranties, condemnation awards, or other tenants, other than through their own share of Operating Expenses;',
      '(g) salaries and benefits of employees above the level of building manager, and the portion of any employee\'s compensation attributable to work for other buildings;',
      '(h) a property management fee greater than three percent of the gross revenues of the Building;',
      '(i) costs of correcting construction defects in the base building, and costs of removing or remediating hazardous materials present on the date of this Lease;',
      '(j) fines, penalties, and late charges incurred because of Landlord\'s violation of law or late payment;',
      '(k) charitable and political contributions, art, and costs of the ownership entity such as accounting for partners and tax returns;',
      '(l) reserves for future expenses, bad debts, and rent losses;',
      '(m) costs of operating the garage to the extent covered by parking charges, and costs of any retail, restaurant, or fitness facility operated for profit; and',
      '(n) any cost that would otherwise be counted twice.',
      `If the Building is less than ninety-five percent occupied in any year, variable Operating Expenses for that year will be adjusted to what they would have been had the Building been ninety-five percent occupied, so that ${c.tenant} pays its share of costs that vary with occupancy on a consistent basis from year to year.`,
    ]],
    ['EXHIBIT G - BUILDING SERVICE HOURS AND AFTER-HOURS RATES', [
      'Building Hours: 7:00 a.m. to 7:00 p.m. on weekdays and 8:00 a.m. to 1:00 p.m. on Saturdays, excluding New Year\'s Day, Memorial Day, Independence Day, Labor Day, Thanksgiving Day and the following day, and Christmas Day.',
      'Heating, ventilation, and air conditioning are supplied during Building Hours at no additional charge. After-hours service is available on four hours\' notice through the tenant service portal, in minimum blocks of two hours, at $68.00 per hour per floor zone, which Landlord may increase once a year to reflect its actual cost of electricity, wear and tear, and supervision.',
      'The supplemental cooling unit serving the server room runs at all times and is separately metered. Tenant will pay for its electricity at the utility\'s rate without markup, together with the cost of a quarterly maintenance contract with a contractor Landlord approves.',
      'Passenger elevators run at all times, with at least one car serving each floor after Building Hours. The freight elevator is available by reservation from 6:00 a.m. to 8:00 a.m. and from 6:00 p.m. to 10:00 p.m. on weekdays, and at any time on weekends, at $55.00 per hour with an operator.',
      'Electricity for lighting and convenience outlets is included in Base Rent up to an average connected load of four watts per rentable square foot. Landlord may install a submeter at Tenant\'s cost if it reasonably believes Tenant\'s use exceeds that level, and Tenant will pay for the excess at cost.',
    ]],
    ['EXHIBIT H - PARKING RULES', [
      `Tenant\'s employees may park in unreserved spaces in the Building garage, up to the number of spaces stated in the Basic Lease Information, at the monthly rate the garage operator charges from time to time, which during the first Lease Year is $145.00 per space. Parking passes are issued by the garage office.`,
      'Visitors may park in the visitor area on the first level at the posted hourly rates. Tenant may buy validation stickers in books of fifty at a twenty percent discount.',
      'Cars must be parked within painted lines, may not be left in the garage for more than seventy-two consecutive hours without notice to the garage office, and may not be washed, repaired, or serviced in the garage except by the operator\'s car wash concession.',
      'Electric vehicle charging stations on the second level are available on a first-come basis at the posted per-kilowatt-hour rate. Vehicles must be moved within thirty minutes after charging is complete.',
      'The garage is open from 6:00 a.m. to 10:00 p.m. on weekdays. Monthly pass holders may enter at other times with their access cards. Landlord is not responsible for loss of or damage to vehicles or their contents except to the extent caused by its negligence.',
    ]],
    ['EXHIBIT I - FORM OF COMMENCEMENT DATE MEMORANDUM', [
      `This Commencement Date Memorandum is delivered under the Office Lease between ${c.landlord}, as Landlord, and ${c.tenant}, as Tenant, for ${c.suite} of ${c.building}.`,
      `1. The Premises were delivered to Tenant with the Landlord Work substantially complete, and Tenant has accepted them, subject only to the punch-list items attached to this memorandum.`,
      `2. The Commencement Date of the Lease is ${c.start}, and the Expiration Date is ${c.end}, unless the Lease is sooner terminated or extended under its terms.`,
      `3. The rentable area of the Premises is ${c.area} square feet and Tenant\'s Share is ${c.share}.`,
      '4. The first monthly installment of Base Rent was paid on signing of the Lease, and the Security Deposit has been delivered in full.',
      '5. Except as stated in this memorandum, the Lease is unmodified and remains in full force and effect. If this memorandum conflicts with the Lease, the Lease controls, except as to the dates and areas stated here.',
      'Each party will sign and return this memorandum within ten business days after the other delivers it. A party\'s failure to sign does not affect the dates stated.',
    ]],
    ['EXHIBIT J - FORM OF TENANT ESTOPPEL CERTIFICATE', [
      `The undersigned Tenant certifies to Landlord, to any prospective purchaser of ${c.building}, and to any lender secured by it, as follows, knowing that they will rely on these statements:`,
      '1. Tenant is the tenant under the Lease described above, which is in full force and effect and has not been amended except as listed in this certificate. A true copy of the Lease and each amendment is attached.',
      '2. Tenant is in possession of the Premises and has not assigned the Lease or sublet any part of the Premises, except as listed in this certificate.',
      '3. Base Rent and Tenant\'s Share of Operating Expenses and Taxes have been paid through the current month, and no rent has been paid more than one month in advance.',
      '4. To Tenant\'s knowledge, Landlord is not in default under the Lease, and no event has occurred that, with notice or the passage of time or both, would be a default by Landlord. Tenant has no claim of offset or defense against the payment of rent.',
      '5. All tenant improvements required of Landlord under the Lease have been completed, and the tenant improvement allowance has been paid in full, except as listed in this certificate.',
      '6. Tenant has no option to purchase the Building or any part of it, and its only rights to extend the Term or to lease additional space are those stated in the Lease.',
      '7. The amount of the Security Deposit held by Landlord is the amount stated in the Lease.',
      '8. Tenant has not received notice of any condemnation, and has not filed or been the subject of any bankruptcy or insolvency proceeding.',
      'The person signing this certificate for Tenant is authorized to do so. Tenant will deliver a completed certificate within ten business days after Landlord requests it, but not more than twice in any twelve-month period.',
    ]],
    ['EXHIBIT K - SUSTAINABILITY PROVISIONS', [
      `${c.building} is operated under a building-wide energy and water efficiency program. Landlord will share with Tenant, at least once a year, the Building\'s energy and water consumption and its benchmarking score, and Tenant will provide its separately metered consumption data on request so that Landlord can complete required disclosures.`,
      'Tenant will use the Building\'s recycling and composting program, will specify low-emitting paints, adhesives, and carpets for any Alterations, and will use lighting with occupancy sensors and daylight controls in any area it renovates.',
      'Landlord may make improvements to reduce the Building\'s energy or water use, and their cost may be included in Operating Expenses as provided in Exhibit F. Landlord will schedule the work to minimize interference with Tenant\'s business.',
      'Neither party will be in default for failure to meet a sustainability goal, but each will cooperate in good faith with the other\'s reasonable requests to improve the Building\'s environmental performance.',
    ]],
  ];
}
