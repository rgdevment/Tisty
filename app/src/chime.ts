type Tone = "filed" | "due";

const NOTES: Record<Tone, { hz: number[]; each: number; peak: number }> = {
  filed: { hz: [880], each: 0.07, peak: 0.05 },
  due: { hz: [660, 880, 1100], each: 0.12, peak: 0.09 },
};

let shared: AudioContext | undefined;

function context(): AudioContext | undefined {
  const Make = window.AudioContext ?? (window as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Make) return undefined;
  shared ??= new Make();
  return shared;
}

/// A sine with an envelope: a bare gate on an oscillator clicks at both ends,
/// and the click is louder than the note.
export function play(tone: Tone): void {
  const audio = context();
  if (!audio) return;
  if (audio.state === "suspended") void audio.resume();

  const note = NOTES[tone];
  note.hz.forEach((hz, step) => {
    const at = audio.currentTime + step * note.each;
    const wave = audio.createOscillator();
    const level = audio.createGain();

    wave.type = "sine";
    wave.frequency.value = hz;
    level.gain.setValueAtTime(0, at);
    level.gain.linearRampToValueAtTime(note.peak, at + 0.01);
    level.gain.exponentialRampToValueAtTime(0.0001, at + note.each);

    wave.connect(level).connect(audio.destination);
    wave.start(at);
    wave.stop(at + note.each + 0.02);
  });
}

export function heard(tone: unknown): tone is Tone {
  return tone === "filed" || tone === "due";
}
