"use client";

import React from "react";
import { VoiceInput } from "@/components/ui/voice-input";
import { AudioUploadCard } from "@/components/ui/audio-upload-card";

export function DemoOne() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 p-6 max-w-5xl mx-auto items-center">
      {/* Audio Upload Card with Unlocked Badge */}
      <div className="relative rounded-2xl border border-border/40 bg-card p-6 shadow-sm">
        <div className="absolute top-4 right-4 z-20">
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-secondary text-secondary-foreground shadow-sm">
            Unlocked
          </span>
        </div>
        <AudioUploadCard
          title="Upload Your Audio"
          description="Drop in your recordings and start transcribing instantly."
        />
      </div>

      {/* Mic Voice Input Card with Unlocked Badge */}
      <div className="relative rounded-2xl border border-border/40 bg-card p-6 shadow-sm min-h-[340px] flex items-center justify-center">
        <div className="absolute top-4 right-4 z-20">
          <span className="px-3 py-1 rounded-full text-xs font-semibold bg-secondary text-secondary-foreground shadow-sm">
            Unlocked
          </span>
        </div>
        <VoiceInput />
      </div>
    </div>
  );
}

export default DemoOne;
