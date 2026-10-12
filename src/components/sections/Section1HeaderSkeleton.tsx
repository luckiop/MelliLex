export const Section1HeaderSkeleton = () => {
  return (
    <div className="animate-pulse">
      <div className="h-12 w-64 bg-muted rounded mb-4"></div>
      
      <div className="flex gap-6 mb-4">
        <div className="h-6 w-32 bg-muted rounded"></div>
        <div className="h-6 w-40 bg-muted rounded"></div>
      </div>

      <div className="h-5 w-48 bg-muted rounded mb-4"></div>

      <div className="flex gap-4 mb-4">
        <div className="h-6 w-24 bg-muted rounded"></div>
        <div className="flex gap-2">
          <div className="h-6 w-20 bg-muted rounded"></div>
          <div className="h-6 w-20 bg-muted rounded"></div>
          <div className="h-6 w-20 bg-muted rounded"></div>
        </div>
      </div>

      <div className="bg-yellow-50 border-2 border-yellow-200 rounded-lg p-4">
        <div className="h-4 w-full bg-muted rounded mb-2"></div>
        <div className="h-4 w-5/6 bg-muted rounded"></div>
      </div>

      <div className="flex gap-3 mt-4">
        <div className="h-9 w-24 bg-muted rounded"></div>
        <div className="h-9 w-24 bg-muted rounded"></div>
      </div>
    </div>
  );
};
