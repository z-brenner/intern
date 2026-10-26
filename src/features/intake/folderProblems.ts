import { contains } from './folderNames';

/**
 * Why a folder cannot be the output folder for `input`, in plain words, or
 * undefined when it can. The backend refuses the same pairs on save
 * (DESTINATION_INSIDE_INTAKE); saying so at the moment of choosing saves a
 * person a trip to the end of setup to find out.
 */
export function outputFolderProblem(input: string, output: string): string | undefined {
  if (!input.trim() || !output.trim()) return undefined;
  if (contains(output, input) && contains(input, output)) return 'The output folder can’t be the same folder as the input folder. Choose a different folder for processed files.';
  if (contains(input, output)) return 'That folder is inside your input folder, so processed files would be picked up and processed again. Choose a folder outside it.';
  return undefined;
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
    case 'FILED_FOLDER_UNAVAILABLE': return 'Intern could not make an Output folder there. Choose another folder for processed files. (FILED_FOLDER_UNAVAILABLE)';
    case 'INBOX_FOLDER_UNAVAILABLE': return 'Intern could not make an Input folder there. Choose another folder. (INBOX_FOLDER_UNAVAILABLE)';
    default: return undefined;
  }
}
