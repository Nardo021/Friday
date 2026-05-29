import { useEffect, useRef, useState } from "react";
import { Mic, Square } from "lucide-react";

import { Button } from "@/components/ui/button";
import { InputGroupButton } from "@/components/ui/input-group";
import { cn } from "@/lib/utils";
import { transcribeAudio } from "@/lib/tauri";

export function VoiceRecorder({
  disabled,
  onTranscript,
  onListeningChange,
  variant = "default",
}: {
  disabled?: boolean;
  onTranscript: (text: string) => void;
  onListeningChange?: (listening: boolean) => void;
  variant?: "default" | "icon";
}) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mediaRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animationRef = useRef<number | null>(null);
  const [levels, setLevels] = useState<number[]>([0, 0, 0, 0, 0, 0]);

  useEffect(() => {
    return () => {
      if (animationRef.current) cancelAnimationFrame(animationRef.current);
      mediaRef.current?.stop();
    };
  }, []);

  const tickWaveform = () => {
    const analyser = analyserRef.current;
    if (!analyser) return;
    const data = new Uint8Array(analyser.frequencyBinCount);
    analyser.getByteFrequencyData(data);
    const slice = Math.floor(data.length / 6);
    const next = Array.from({ length: 6 }, (_, i) => {
      const v = data[i * slice] ?? 0;
      return Math.max(4, Math.round((v / 255) * 24));
    });
    setLevels(next);
    animationRef.current = requestAnimationFrame(tickWaveform);
  };

  const stopRecording = () => {
    mediaRef.current?.stop();
    mediaRef.current = null;
    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current);
      animationRef.current = null;
    }
    setRecording(false);
    onListeningChange?.(false);
    setLevels([0, 0, 0, 0, 0, 0]);
  };

  const startRecording = async () => {
    if (disabled || recording) return;
    setError(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const audioCtx = new AudioContext();
      const source = audioCtx.createMediaStreamSource(stream);
      const analyser = audioCtx.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);
      analyserRef.current = analyser;

      const recorder = new MediaRecorder(stream);
      chunksRef.current = [];
      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };
      recorder.onstop = () => {
        stream.getTracks().forEach((t) => t.stop());
        void audioCtx.close();
        const blob = new Blob(chunksRef.current, { type: "audio/webm" });
        void blob.arrayBuffer().then((buf) => {
          const bytes = new Uint8Array(buf);
          let binary = "";
          for (let i = 0; i < bytes.length; i++) {
            binary += String.fromCharCode(bytes[i]!);
          }
          const base64 = btoa(binary);
          void transcribeAudio(base64)
            .then((r) => onTranscript(r.transcript))
            .catch((e: unknown) =>
              setError(e instanceof Error ? e.message : "Transcription failed"),
            );
        });
      };
      mediaRef.current = recorder;
      recorder.start();
      setRecording(true);
      onListeningChange?.(true);
      animationRef.current = requestAnimationFrame(tickWaveform);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Microphone access denied");
    }
  };

  if (variant === "icon") {
    return (
      <div className="flex items-center">
        <InputGroupButton
          size="icon-sm"
          disabled={disabled}
          aria-label={recording ? "Stop voice input" : "Start voice input"}
          className={cn(recording && "text-destructive hover:text-destructive")}
          onClick={() => (recording ? stopRecording() : void startRecording())}
        >
          {recording ? (
            <Square data-icon="inline-start" className="fill-current" />
          ) : (
            <Mic data-icon="inline-start" />
          )}
        </InputGroupButton>
        {error && (
          <span className="sr-only" role="alert">
            {error}
          </span>
        )}
      </div>
    );
  }

  return (
    <div className="flex items-center gap-1">
      <Button
        type="button"
        size="sm"
        variant={recording ? "destructive" : "secondary"}
        className="min-h-9 px-3 text-xs"
        disabled={disabled}
        aria-label={recording ? "Stop voice input" : "Start voice input"}
        onClick={() => (recording ? stopRecording() : void startRecording())}
        onPointerDown={(e) => {
          if (e.button === 0 && e.shiftKey) {
            e.preventDefault();
            void startRecording();
          }
        }}
        onPointerUp={(e) => {
          if (e.shiftKey && recording) stopRecording();
        }}
        title="Click to record · Shift+hold for push-to-talk"
      >
        {recording ? (
          <Square data-icon="inline-start" className="fill-current" />
        ) : (
          <Mic data-icon="inline-start" />
        )}
      </Button>
      {recording && (
        <div className="flex h-4 items-end gap-0.5">
          {levels.map((h, i) => (
            <div
              key={i}
              className="w-0.5 rounded-sm bg-primary"
              style={{ height: `${h}px` }}
            />
          ))}
        </div>
      )}
      {error && <span className="text-[10px] text-destructive">{error}</span>}
    </div>
  );
}
