import { ErrorBoundary } from "./ErrorBoundary";
import { WindowRouter } from "./routes";

export default function App() {
  return (
    <ErrorBoundary>
      <WindowRouter />
    </ErrorBoundary>
  );
}
