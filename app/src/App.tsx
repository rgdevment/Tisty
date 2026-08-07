import { useCallback, useEffect, useState } from "react";
import { capture, complete, snapshot, type Snapshot } from "./core";

export default function App() {
  const [data, setData] = useState<Snapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [text, setText] = useState("");

  const load = useCallback(() => {
    snapshot().then(setData).catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    load();
    // The CLI writes to the same store while this window is open.
    window.addEventListener("focus", load);
    return () => window.removeEventListener("focus", load);
  }, [load]);

  const write = (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    capture(text)
      .then(() => {
        setText("");
        load();
      })
      .catch((e) => setError(String(e)));
  };

  if (!data) return null;

  return (
    <>
      <form onSubmit={write}>
        <input
          autoFocus
          className="m-2 w-96 rounded border px-2 py-1"
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </form>
      {error && <p>{error}</p>}
      <ul>
        {data.tasks.map((task) => (
          <li key={task.id}>
            <button onClick={() => complete(task.id).then(load).catch((e) => setError(String(e)))}>done</button>
            {task.title}
          </li>
        ))}
      </ul>
    </>
  );
}
