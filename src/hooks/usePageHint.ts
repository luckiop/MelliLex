import { useEffect, useState } from 'react';
import { PageHintService, type PageHintId } from '../services/PageHintService';

export function usePageHint(pageId: PageHintId) {
  const [visible, setVisible] = useState(() => PageHintService.peek(pageId));

  useEffect(() => {
    setVisible(PageHintService.recordVisit(pageId));
  }, [pageId]);

  const dismiss = () => {
    PageHintService.dismiss(pageId);
    setVisible(false);
  };

  return { visible, dismiss };
}
