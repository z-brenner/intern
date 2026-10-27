import { contains } from './folderNames';
import type { CloudLocation } from '../../types';

const SAME_FOLDER = 'The output folder can’t be the same folder as the input folder. Choose a different folder for processed files.';

/**
 * Why a folder cannot be the output folder for `input`, in plain words, or
 * undefined when it can. Neither folder may hold the other, the rule the
 * backend enforces on save (DESTINATION_INSIDE_INTAKE,
 * INTAKE_INSIDE_DESTINATION); saying so at the moment of choosing saves a
 * person a trip to the end of setup to find out.
 */
export function outputFolderProblem(input: string, output: string): string | undefined {
  if (!input.trim() || !output.trim()) return undefined;
  if (contains(output, input) && contains(input, output)) return SAME_FOLDER;
  if (contains(input, output)) return 'That folder is inside your input folder, so processed files would be picked up and processed again. Choose a folder outside it.';
  if (contains(output, input)) return 'Your input folder is inside that folder. Choose an output folder outside it - neither folder may hold the other.';
  return undefined;
}

/** The same rule, said about a new input folder for the output folder already chosen. */
export function inputFolderProblem(input: string, output: string): string | undefined {
  if (!input.trim() || !output.trim()) return undefined;
  if (contains(output, input) && contains(input, output)) return SAME_FOLDER;
  if (contains(input, output)) return 'Your output folder is inside that folder, so processed files would be picked up and processed again. Choose another input folder, or change the output folder first.';
  if (contains(output, input)) return 'That folder is inside your output folder, where processed files are filed. Choose an input folder outside it.';
  return undefined;
}

/**
 * Whether description records belong with an output folder in `location`: a
 * SharePoint library, or OneDrive for work or school, which keeps one. A
 * personal OneDrive has no SharePoint Description column to fill.
 */
export function keepsDescriptionColumn(location: CloudLocation | null | undefined): boolean {
  return location?.provider === 'sharepoint' || location?.provider === 'onedrive_business';
}

/**
 * The backend's refusals of a pair of folders, in the same plain words, or
 * undefined for any other code. Each keeps its code in brackets for the
 * guide's troubleshooting table.
 */
export function folderSaveProblem(code: string | undefined): string | undefined {
  switch (code?.trim()) {
    case 'INTAKE_FOLDER_MISSING': return 'Intern could not find the input folder. Choose a folder that exists. (INTAKE_FOLDER_MISSING)';
    case 'INTAKE_NEEDS_DESTINATION': return 'Choose an output folder too. Intern needs both: an input folder for files to process, and an output folder outside it for processed files. (INTAKE_NEEDS_DESTINATION)';
    case 'DESTINATION_INSIDE_INTAKE': return 'The output folder is inside the input folder (or is the same folder), so processed files would be picked up and processed again. Choose an output folder outside it. (DESTINATION_INSIDE_INTAKE)';
    case 'INTAKE_INSIDE_DESTINATION': return 'The input folder is inside the output folder. Choose folders where neither holds the other. (INTAKE_INSIDE_DESTINATION)';
    case 'FILED_FOLDER_UNAVAILABLE': return 'Intern could not make an Output folder there. Choose another folder for processed files. (FILED_FOLDER_UNAVAILABLE)';
    case 'INBOX_FOLDER_UNAVAILABLE': return 'Intern could not make an Input folder there. Choose another folder. (INBOX_FOLDER_UNAVAILABLE)';
    default: return undefined;
  }
}
