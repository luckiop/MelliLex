import { Star } from 'lucide-react';

interface Props {
  count: number;
  className?: string;
}

export function VocabStarIcons({ count, className = 'w-4 h-4' }: Props) {
  const stars = Math.max(0, Math.min(count, 2));
  if (stars === 0) {
    return <Star className={className} />;
  }

  return (
    <span className="inline-flex items-center">
      {Array.from({ length: stars }, (_, index) => (
        <Star key={index} className={`${className} fill-current`} />
      ))}
    </span>
  );
}
