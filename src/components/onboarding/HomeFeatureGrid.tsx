import {
  BookMarked,
  Cpu,
  FileText,
  History,
  Settings,
  Sparkles,
} from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Card } from '../ui/card';

const FEATURES = [
  { id: 'define', icon: Sparkles },
  { id: 'vocab', icon: BookMarked },
  { id: 'notes', icon: FileText },
  { id: 'technical', icon: Cpu },
  { id: 'history', icon: History },
  { id: 'settings', icon: Settings },
] as const;

export function HomeFeatureGrid() {
  const { t } = useTranslation();

  return (
    <div className="mt-10 grid grid-cols-2 gap-3 text-left">
      {FEATURES.map(({ id, icon: Icon }) => (
        <Card key={id} className="cursor-default px-3.5 py-3 shadow-none">
          <Icon className="mb-2 h-4 w-4 text-muted-foreground" aria-hidden />
          <p className="text-sm font-medium text-foreground">
            {t(`home.features.${id}.title`)}
          </p>
          <p className="mt-1 text-xs leading-relaxed text-muted-foreground">
            {t(`home.features.${id}.desc`)}
          </p>
        </Card>
      ))}
    </div>
  );
}
