import React from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TriangleAlert } from "lucide-react";

interface Props {
  children: React.ReactNode;
}

interface State {
  error: Error | null;
}

function windowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "unknown";
  }
}

export class ErrorBoundary extends React.Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  render() {
    if (this.state.error) {
      return (
        <div className="flex h-screen flex-col items-center justify-center gap-3 bg-background px-6 text-center text-foreground">
          <TriangleAlert className="size-10 text-destructive" />
          <h1 className="text-lg font-semibold">Friday failed to load</h1>
          <p className="text-xs text-muted-foreground">Window: {windowLabel()}</p>
          <p className="max-w-md text-sm text-muted-foreground">
            {this.state.error.message}
          </p>
        </div>
      );
    }

    return this.props.children;
  }
}
