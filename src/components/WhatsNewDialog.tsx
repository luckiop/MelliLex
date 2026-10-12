import { useTranslation } from 'react-i18next';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from './ui/dialog';
import { Button } from './ui/button';

interface WhatsNewDialogProps {
  open: boolean;
  version: string;
  onDismiss: () => void;
}

export function WhatsNewDialog({ open, version, onDismiss }: WhatsNewDialogProps) {
  const { t } = useTranslation();
  const items = t('whatsNew.items', { returnObjects: true });
  const notes = Array.isArray(items) ? items.filter((item) => typeof item === 'string') : [];

  return (
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onDismiss(); }}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('whatsNew.title', { version })}</DialogTitle>
          <DialogDescription>{t('whatsNew.subtitle')}</DialogDescription>
        </DialogHeader>
        <ul className="space-y-2 text-sm leading-relaxed text-foreground/90 list-disc pl-5">
          {notes.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
        <DialogFooter>
          <Button onClick={onDismiss}>{t('whatsNew.gotIt')}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
