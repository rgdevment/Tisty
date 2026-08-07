import { useCallback, useEffect, useState } from "react";
import { complete, snapshot, type Snapshot } from "./core";

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(() => {
    snapshot().then(setData).catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
    // The CLI writes to the same store while this window is open.
    window.addEventListener("focus", load);
    return () => window.removeEventListener("focus", load);
  }, [load]);

  if (error) return <p>{error}</p>;
  if (!data) return null;

  return (
    <ul>
      {data.tasks.map((task) => (
        <li key={task.id}>
          <button onClick={() => complete(task.id).then(load).catch((e) => setError(String(e)))}>done</button>
          {task.title}
        </li>
      ))}
    </ul>
  );
}
