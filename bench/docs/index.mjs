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
import { agreementTwoColumnFootnotes, meetingNoticeColumns, meetingNoticeInterleaved, meetingNoticeReversed, newsletterThreeColumn, rateConfirmationRotated } from './columns.mjs';
import { benefitsChangeForm, inspectionLogTwoPages, invoiceBoxedGrid, invoiceLabelAbove, invoiceRightAligned, lossNoticeBoxedFields, priceListUnruled } from './grids.mjs';
import { mixedSignatureRegion, scanBillOfLading, scanCancellationNotice150, scanCertificateOfInsurance, scanMixedMiddlePage, scanRemittanceAdvice120, scanRotatedPageInPdf } from './critical.mjs';
import { masterServicesAgreement12 } from './long-msa.mjs';
import { industrialLease25 } from './long-lease-schedule.mjs';
import { termLoan40 } from './long-loan.mjs';
import { propertyPolicy60 } from './long-policy.mjs';
import { watershedMonitoringReport100 } from './long-monitoring.mjs';
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
  // Added for the structure measurements; recorded live later.
  ['newsletter-three-column', newsletterThreeColumn],
  ['agreement-two-column-footnotes', agreementTwoColumnFootnotes],
  ['meeting-notice-columns', meetingNoticeColumns],
  ['meeting-notice-interleaved', meetingNoticeInterleaved],
  ['meeting-notice-reversed', meetingNoticeReversed],
  ['rate-confirmation-rotated', rateConfirmationRotated],
  ['inspection-log-ruled-2p', inspectionLogTwoPages],
  ['price-list-unruled', priceListUnruled],
  ['invoice-label-above', invoiceLabelAbove],
  ['invoice-right-aligned', invoiceRightAligned],
  ['invoice-boxed-grid', invoiceBoxedGrid],
  ['benefits-change-checkbox-form', benefitsChangeForm],
  ['loss-notice-boxed-fields', lossNoticeBoxedFields],
  ['scan-rotated-page-in-pdf', scanRotatedPageInPdf],
  ['scan-cancellation-notice-150dpi', scanCancellationNotice150],
  ['scan-remittance-advice-120dpi', scanRemittanceAdvice120],
  ['scan-mixed-middle-page', scanMixedMiddlePage],
  ['mixed-signature-region', mixedSignatureRegion],
  ['scan-certificate-of-insurance', scanCertificateOfInsurance],
  ['scan-bill-of-lading', scanBillOfLading],
  // Added for phase 3: long documents whose deciding evidence sits deep
  // inside, far from the first page; recorded live later.
  ['msa-effective-date-in-definitions-12p', masterServicesAgreement12],
  ['industrial-lease-dated-in-schedule-25p', industrialLease25],
  ['term-loan-parties-apart-40p', termLoan40],
  ['property-policy-declarations-mid-60p', propertyPolicy60],
  ['watershed-monitoring-report-100p', watershedMonitoringReport100],
];
