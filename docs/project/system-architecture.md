# **Audio Cue System: Core Architecture & Concurrency Model**

This document defines the highly concurrent, strict **Model-View-Controller (MVC)** paradigm executed across three distinct processing domains.

*Update:* The system has been consolidated into three primary domains. The Audio Engine acts as a self-contained, autonomous subsystem that completely encapsulates the high-priority CPAL threads, asset decoders, and event forwarders, allowing the Execution Thread to operate purely as a fire-and-forget orchestrator.

## **1\. The Three-Domain Architecture**

### **Domain 1: The UI Thread (View & Context)**

* **Type:** OS Main Thread.  
* **Libraries:** floem (GUI), serde (File I/O).  
* **Domain Ownership:**  
  * Application rendering and keyboard/mouse interaction.  
  * **The Source of Truth:** Owns and mutates the canonical WorkspaceState via ArcSwap.  
  * **Telemetry Polling:** Drives the visual VU meters and progress bars by synchronously polling the Audio Engine's lock-free telemetry ringbuffer at 60 FPS.

### **Domain 2: The Execution Engine (Controller / Orchestrator)**

* **Type:** Primary Background Async Runtime.  
* **Libraries:** tokio (Multi-thread runtime).  
* **Domain Ownership:**  
  * **The Playhead & Logic:** Orchestrates the Async Actors representing Cue lifecycles.  
  * **Event Driven:** Awaits incoming UserIntent (from UI) and AudioEvent (from the Audio Engine).  
  * **Fire-and-Forget:** Translates logical intent into a single, high-level DSPCommand::Play(CueId, PlaybackClip) and releases ownership of the audio lifecycle.

### **Domain 3: The Autonomous Audio Engine Subsystem**

* **Type:** Hybrid (CPAL Real-Time OS Thread \+ Isolated Tokio Runtime).  
* **Libraries:** cpal, rodio::Decoder, ringbuf, tokio::sync::Notify.  
* **Sub-Threads Contained:**  
  * **3A. CPAL Audio Thread (Muscle):** High-priority OS thread. Reads lock-free PCM buffers, applies volume envelopes, executes matrix routing, and pushes events/telemetry.  
  * **3B. Asset Decoder Tasks:** Tokio blocking tasks. Handles Disk I/O and MP3/WAV decoding, feeding lock-free PCM ringbuffers.  
  * **3C. Event Forwarder Task:** A lightweight Tokio task that sleeps using 0% CPU via an **Atomic Waker (Notify)**. When the CPAL thread finishes a cue, it wakes this task to safely forward the event to the Execution Engine.

## **2\. Inter-Domain Communication Boundaries**

The system relies on specific communication primitives tailored to the latency and thread-safety requirements of each boundary.

### **A. UI ⟷ Execution Engine**

* ArcSwap\<WorkspaceState\> **(UI ➔ Exec):** Zero-lock data handoff of the entire show file.  
* mpsc::Sender\<UserIntent\> **(UI ➔ Exec):** Asynchronous push of discrete user commands (GO, Panic).  
* watch::Receiver\<ExecutionState\> **(Exec ➔ UI):** State broadcasting (active cues, playhead). The UI reactively updates without queuing backlog.

### **B. Execution Engine ⟷ Audio Engine**

* mpsc::Sender\<DSPCommand\> **(Exec ➔ Audio):** Asynchronous push of high-level audio intents. The Audio Engine router intercepts this and provisions the internal CPAL/Decoder components.  
* mpsc::Receiver\<AudioEvent\> **(Audio ➔ Exec):** Asynchronous return path for structural triggers (PlaybackFinished). Safely bridged from the CPAL thread via the Event Forwarder task.

### **C. Audio Engine ➔ UI (The Direct Telemetry Path)**

* ringbuf::Consumer\<AudioTelemetry\> **(Audio ➔ UI):** High-frequency metrics (VU meters, playhead progress). Bypasses the Execution engine entirely. Polled synchronously by the UI render loop.

## **3\. Internal Audio Engine Pipeline (The "Black Box")**

Inside Domain 3, the Audio Engine manages its own complex real-time synchronization so the rest of the application doesn't have to.

graph TD  
    subgraph Audio Engine Domain  
        Router\[Audio Engine Router (Tokio)\]  
        Decoder\[Asset Decoder Task (Tokio Blocking)\]  
        CPAL\[CPAL DSP Node (Real-Time OS Thread)\]  
        Forwarder\[Event Forwarder (Tokio)\]

        Router \-- Spawns \--\> Decoder  
        Router \-- Allocates Node \--\> CPAL  
          
        Decoder \-- "PCM Float Samples (Ringbuf)" \--\> CPAL  
        CPAL \-- "DecoderControl::SeekTo (Ringbuf)" \--\> Decoder  
          
        CPAL \-- "Notify::notify\_one() \+ AudioEvent (Ringbuf)" \--\> Forwarder  
    end  
      
    Forwarder \-- "mpsc::send(AudioEvent)" \--\> Exec\[Execution Domain\]  
    CPAL \-- "AudioTelemetry (Ringbuf)" \--\> UI\[UI Domain\]

### **The Loop & Wrap Handshake**

When the CPAL thread detects that the current frame matches a trim\_end\_sec limit on a looped track, it instantly flushes the PCM ringbuffer and pushes DecoderControl::SeekTo back to the Decoder task. The CPAL thread outputs silence for a fraction of a millisecond while the decoder reseeds the buffer, guaranteeing perfect sample-accurate loop wrapping without Execution Engine oversight.