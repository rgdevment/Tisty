type Tone = "filed" | "done" | "due";

type Shape = { hz: number[]; apart: number; tail: number; peak: number };

const NOTES: Record<Tone, Shape> = {
  filed: { hz: [523.25, 659.25, 783.99], apart: 0.05, tail: 0.42, peak: 0.045 },
  done: { hz: [523.25, 659.25, 783.99, 1046.5], apart: 0.07, tail: 0.55, peak: 0.05 },
  due: { hz: [660, 880, 1100], apart: 0.12, tail: 0.12, peak: 0.09 },
};

let shared: AudioContext | undefined;

function context(): AudioContext | undefined {
  const Make =
    window.AudioContext ??
    (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Make) return undefined;
  shared ??= new Make();
  return shared;
}

export function play(tone: Tone): void {
  const audio = context();
  if (!audio) return;
  if (audio.state === "suspended") void audio.resume();

  const note = NOTES[tone];
  note.hz.forEach((hz, step) => {
    const at = audio.currentTime + step * note.apart;
    const wave = audio.createOscillator();
    const level = audio.createGain();

    wave.type = "sine";
    wave.frequency.value = hz;
    level.gain.setValueAtTime(0, at);
    level.gain.linearRampToValueAtTime(note.peak, at + 0.012);
    level.gain.exponentialRampToValueAtTime(0.0001, at + note.tail);

    wave.connect(level).connect(audio.destination);
    wave.start(at);
    wave.stop(at + note.tail + 0.02);
  });
}

export function heard(tone: unknown): tone is Tone {
  return tone === "filed" || tone === "done" || tone === "due";
}
