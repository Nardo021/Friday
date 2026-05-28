import React from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
        <div className="flex h-screen flex-col items-center justify-center gap-3 bg-zinc-950 px-6 text-center text-zinc-200">
          <h1 className="text-lg font-semibold">Friday failed to load</h1>
          <p className="text-xs text-zinc-500">Window: {windowLabel()}</p>
          <p className="max-w-md text-sm text-zinc-400">
            {this.state.error.message}
          </p>
        </div>
      );
    }

    return this.props.children;
  }
}
