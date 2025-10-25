export function LoadingSkeleton() {
    return (
      <div className="space-y-6 animate-pulse">
        {[1, 2, 3].map((i) => (
          <div key={i} className="glass-card rounded-2xl p-6 h-48">
            <div className="h-6 bg-white/10 rounded w-1/3 mb-4"></div>
            <div className="space-y-3">
              <div className="h-4 bg-white/10 rounded"></div>
              <div className="h-4 bg-white/10 rounded w-5/6"></div>
            </div>
          </div>
        ))}
      </div>
    );
  }