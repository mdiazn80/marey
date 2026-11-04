import { useCallback, useState } from "react";

/** Runs one-shot actions and keeps a short message about how the latest one went. */
export function useActionFeedback() {
  const [feedback, setFeedback] = useState("");
  const run = useCallback((action: () => Promise<string>) => {
    action().then(setFeedback, (error) => setFeedback(String(error)));
  }, []);
  return { feedback, run };
}
