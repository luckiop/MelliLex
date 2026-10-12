import { useEffect, useState } from 'react';
import { getVersion } from '@tauri-apps/api/app';
import { WhatsNewDialog } from '../components/WhatsNewDialog';
import { WhatsNewService } from '../services/WhatsNewService';

export function WhatsNewBridge() {
  const [version, setVersion] = useState('');
  const [open, setOpen] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void getVersion()
      .then((value) => {
        if (cancelled) {
          return;
        }
        setVersion(value);
        setOpen(WhatsNewService.shouldShow(value));
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  const dismiss = () => {
    if (version) {
      WhatsNewService.markSeen(version);
    }
    setOpen(false);
  };

  return <WhatsNewDialog open={open} version={version} onDismiss={dismiss} />;
}
