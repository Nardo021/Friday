import { AppProviders } from "./providers";
import { ErrorBoundary } from "./ErrorBoundary";
import { WindowRouter } from "./routes";

export default function App() {
  return (
    <AppProviders>
      <ErrorBoundary>
        <WindowRouter />
      </ErrorBoundary>
    </AppProviders>
  );
}
