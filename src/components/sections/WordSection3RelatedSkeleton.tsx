export const WordSection3RelatedSkeleton = () => {
  return (
    <div className="animate-pulse space-y-6">
      <div>
        <div className="h-6 w-32 bg-muted rounded mb-3"></div>
        <div className="flex flex-wrap gap-2">
          {[1, 2, 3, 4, 5, 6].map((i) => (
            <div key={i} className="h-8 w-24 bg-muted rounded-full"></div>
          ))}
        </div>
      </div>

      <div>
        <div className="h-6 w-32 bg-muted rounded mb-3"></div>
        <div className="flex flex-wrap gap-2">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="h-8 w-28 bg-muted rounded-full"></div>
          ))}
        </div>
      </div>

      <div>
        <div className="h-6 w-48 bg-muted rounded mb-3"></div>
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="bg-blue-50 border border-blue-200 rounded-lg p-3">
              <div className="h-5 w-40 bg-muted rounded mb-2"></div>
              <div className="h-4 w-full bg-muted rounded"></div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
