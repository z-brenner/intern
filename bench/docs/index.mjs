/// Every InternBench document, in corpus order: [id, builder].
import { capitalCallNotice, noticeOfDefault, noticeRentIncrease } from './notices.mjs';
import { invoiceDateInTable, invoiceLayoutOnly, purchaseOrder } from './invoices.mjs';
import { assignmentAssumption, leaseTwoColumn, promissoryNote, secondAmendment, servicesAgreement, sowUnderMsa } from './contracts.mjs';
import { engagementLetter, letterheadLetter, offerLetter } from './letters.mjs';
import { accountStatement, explanationOfBenefits, priorAuthorization } from './statements.mjs';
import { annualReportExcerpt, declarationsInterleaved } from './complex.mjs';
import { changeOrderForm, vendorRegistrationForm } from './forms.mjs';
import { aircraftMaintenanceLog, boardMinutes, fieldConditionReport } from './records.mjs';
import { apAgingReport, boardResolution, demandLetter, harvestLog, payrollRegister, productLaunchPlan, quarterlyBusinessReview, separationAgreement } from './officefiles.mjs';
import { courtHearingNotice, emailApprovalThread } from './messages.mjs';
import { dataProcessingAgreement } from './long-dpa.mjs';
import { assetPurchaseAgreement } from './long-apa.mjs';
import { creditAgreement } from './long-credit.mjs';
import { annualReport } from './long-annual.mjs';
import { ocrCorruptedInvoice, scanAgreement, scanCleanLease, scanFaintLetter, scanFaxTwoFrames, scanLease, scanLowResReceipt, scanMixedAmendment, scanNoisyStatement, scanPatientIntakeForm, scanRotatedInvoice, scanSkewedNotice, scanUpsideDownPo } from './scans.mjs';

export const BUILDERS = [
  ['notice-rent-increase', noticeRentIncrease],
  ['invoice-date-in-table', invoiceDateInTable],
  ['invoice-layout-only', invoiceLayoutOnly],
  ['account-statement', accountStatement],
  ['purchase-order', purchaseOrder],
  ['services-agreement', servicesAgreement],
  ['second-amendment', secondAmendment],
  ['sow-under-msa-5p', sowUnderMsa],
  ['notice-of-default', noticeOfDefault],
  ['offer-letter', offerLetter],
  ['prior-authorization', priorAuthorization],
  ['explanation-of-benefits', explanationOfBenefits],
  ['promissory-note', promissoryNote],
  ['capital-call-notice', capitalCallNotice],
  ['engagement-letter', engagementLetter],
  ['letterhead-letter', letterheadLetter],
  ['lease-two-column', leaseTwoColumn],
  ['declarations-interleaved', declarationsInterleaved],
  ['annual-report-excerpt-8p', annualReportExcerpt],
  ['vendor-registration-form', vendorRegistrationForm],
  ['change-order-form', changeOrderForm],
  ['board-minutes', boardMinutes],
  ['field-condition-report', fieldConditionReport],
  ['aircraft-maintenance-log', aircraftMaintenanceLog],
  ['assignment-assumption', assignmentAssumption],
  ['data-processing-agreement-10p', dataProcessingAgreement],
  ['asset-purchase-agreement-25p', assetPurchaseAgreement],
  ['credit-agreement-50p', creditAgreement],
  ['annual-report-100p', annualReport],
  ['separation-agreement', separationAgreement],
  ['demand-letter', demandLetter],
  ['board-resolution', boardResolution],
  ['quarterly-business-review', quarterlyBusinessReview],
  ['product-launch-plan', productLaunchPlan],
  ['payroll-register', payrollRegister],
  ['ap-aging-report', apAgingReport],
  ['harvest-log', harvestLog],
  ['email-approval-thread', emailApprovalThread],
  ['court-hearing-notice', courtHearingNotice],
  ['scan-clean-lease-2p', scanCleanLease],
  ['scan-mixed-amendment', scanMixedAmendment],
  ['scan-rotated-90-invoice', scanRotatedInvoice],
  ['scan-upside-down-po', scanUpsideDownPo],
  ['scan-skewed-notice', scanSkewedNotice],
  ['scan-low-res-receipt', scanLowResReceipt],
  ['scan-noisy-statement', scanNoisyStatement],
  ['scan-faint-letter', scanFaintLetter],
  ['scan-agreement-10p', scanAgreement],
  ['scan-lease-25p', scanLease],
  ['ocr-corrupted-invoice', ocrCorruptedInvoice],
  ['scan-fax-two-frames', scanFaxTwoFrames],
  ['scan-patient-intake-form', scanPatientIntakeForm],
];
