type SoundType = 'move' | 'capture';

let audioCtx: AudioContext | null = null;
let moveBuffer: AudioBuffer | null = null;
let captureBuffer: AudioBuffer | null = null;

async function initAudio() {
  if (typeof window === 'undefined') return;

  try {
    const AudioContextClass =
      window.AudioContext || (window as any).webkitAudioContext;
    audioCtx = new AudioContextClass();

    const [moveRes, captureRes] = await Promise.all([
      fetch('/assets/sounds/Move.ogg'),
      fetch('/assets/sounds/Capture.ogg')
    ]);

    const moveArrayBuffer = await moveRes.arrayBuffer();
    const captureArrayBuffer = await captureRes.arrayBuffer();

    moveBuffer = await audioCtx.decodeAudioData(moveArrayBuffer);
    captureBuffer = await audioCtx.decodeAudioData(captureArrayBuffer);
  } catch (err) {
    console.warn('Failed to initialize Web Audio API:', err);
  }
}

if (typeof window !== 'undefined') {
  initAudio();
}

export function playBoardSound(type: SoundType = 'move') {
  if (!audioCtx) return;

  if (audioCtx.state === 'suspended') {
    audioCtx.resume();
  }

  const buffer = type === 'capture' ? captureBuffer : moveBuffer;

  if (!buffer) return;

  const source = audioCtx.createBufferSource();
  source.buffer = buffer;

  const gainNode = audioCtx.createGain();
  gainNode.gain.value = 0.6;

  source.connect(gainNode);
  gainNode.connect(audioCtx.destination);

  source.start(0);
}
