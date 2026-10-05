import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { FileKindIcon } from './FileKindIcon';

describe('FileKindIcon', () => {
  it('marks decks, email, and the legacy and open formats by their kind', () => {
    const kinds = {
      'Board deck.pptx': 'presentation',
      'deck.ppt': 'presentation',
      'invoice.msg': 'email',
      'notice.eml': 'email',
      'letter.doc': 'document',
      'ledger.xls': 'spreadsheet',
      'statement.csv': 'text',
      'archive.zip': 'other',
    };
    for (const [filename, kind] of Object.entries(kinds)) {
      const { container, unmount } = render(<FileKindIcon filename={filename} />);
      expect(container.firstElementChild).toHaveClass(`file-kind--${kind}`);
      unmount();
    }
  });
});
