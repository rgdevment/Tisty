// The catalogue is named for what an icon is for — `deal`, `travel`, `sprint` — and people look
// for what it draws, in their own language.
const ALSO: Record<string, string[]> = {
  cara: ["mood", "mood-happy", "mood-sad", "mood-flat", "angry", "annoyed"],
  caras: ["mood", "mood-happy", "mood-sad", "mood-flat", "angry", "annoyed"],
  face: ["mood", "mood-happy", "mood-sad", "mood-flat", "angry", "annoyed"],
  smile: ["mood", "mood-happy"],
  smiley: ["mood", "mood-happy"],
  sonrisa: ["mood", "mood-happy"],
  feliz: ["mood-happy", "mood"],
  happy: ["mood-happy", "mood"],
  risa: ["mood-happy"],
  laugh: ["mood-happy"],
  triste: ["mood-sad"],
  sad: ["mood-sad"],
  frown: ["mood-sad"],
  enfado: ["angry", "annoyed"],
  enojo: ["angry", "annoyed"],
  emocion: ["mood", "mood-happy", "mood-sad", "mood-flat", "angry", "annoyed"],
  emoji: ["mood", "mood-happy", "mood-sad", "mood-flat"],
  emoticon: ["mood", "mood-happy", "mood-sad", "mood-flat"],
  animo: ["mood", "mood-happy", "mood-sad", "mood-flat"],
  mano: ["hand", "fist", "grab", "pointer", "metal", "hand-heart", "physio"],
  manos: ["hand", "fist", "grab", "pointer", "metal", "hand-heart", "deal"],
  gesto: ["hand", "fist", "grab", "pointer", "metal", "thumbs-up", "thumbs-down"],
  gestos: ["hand", "fist", "grab", "pointer", "metal", "thumbs-up", "thumbs-down"],
  gesture: ["hand", "fist", "grab", "pointer", "metal", "thumbs-up", "thumbs-down"],
  puno: ["fist"],
  fist: ["fist"],
  pulgar: ["thumbs-up", "thumbs-down"],
  thumb: ["thumbs-up", "thumbs-down"],
  like: ["thumbs-up", "heart"],
  apreton: ["deal", "loan", "neighbour"],
  handshake: ["deal", "loan", "neighbour"],
  saludo: ["deal", "hand"],
  corazon: ["heart", "heart-thing", "hand-heart", "chat-heart"],
  auto: ["car", "taxi", "rental", "car-battery", "ev-charge"],
  coche: ["car", "taxi", "rental"],
  carro: ["car", "taxi", "rental"],
  vehiculo: ["car", "bus", "taxi", "motorbike", "bike", "truck-electric", "van", "train"],
  vehicle: ["car", "bus", "taxi", "motorbike", "bike", "truck-electric", "van", "train"],
  moto: ["motorbike", "scooter"],
  motorcycle: ["motorbike", "scooter"],
  bici: ["bike", "bike-sport"],
  bicicleta: ["bike", "bike-sport"],
  camion: ["delivery", "vendor", "truck-electric"],
  truck: ["delivery", "vendor", "truck-electric"],
  furgoneta: ["van"],
  avion: ["travel", "flight", "landing", "tickets-plane"],
  plane: ["travel", "flight", "landing", "tickets-plane"],
  vuelo: ["travel", "flight", "landing"],
  barco: ["boat", "ferry", "cruise", "anchor", "ship-wheel"],
  boat: ["boat", "ferry", "cruise"],
  velero: ["boat"],
  tren: ["train", "tram", "metro"],
  train: ["train", "tram", "metro"],
  autobus: ["bus", "bus-front"],
  cohete: ["sprint", "rocket-thing", "launch"],
  rocket: ["sprint", "rocket-thing", "launch"],
  fiesta: ["launch", "cake", "balloon"],
  party: ["launch", "cake", "balloon"],
  casa: ["home", "flat", "house-heart", "room"],
  perro: ["dog"],
  gato: ["cat"],
  familia: ["family", "friends", "baby"],
  gafas: ["glasses", "optician"],
  lentes: ["glasses", "optician"],
};

const plain = (said: string): string =>
  said
    .trim()
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "");

export const alsoNamed = (word: string): string[] => {
  const said = plain(word);
  if (said.length < 2) return [];
  const out: string[] = [];
  for (const [key, named] of Object.entries(ALSO)) {
    if (key.includes(said) || said.includes(key)) out.push(...named);
  }
  return [...new Set(out)];
};

export const everyNamed = (): string[] => [...new Set(Object.values(ALSO).flat())];
