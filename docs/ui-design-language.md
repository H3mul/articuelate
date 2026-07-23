# **Articuelate Design Language: Zed Philosophy**

This document outlines the design language mapping for building a QLab-style interface using the principles of the Zed editor. The core philosophy relies on depth through luminance, separating surfaces from structural boundaries, and using "punch-down" interactables.

## **Core Principles**

1. **Surfaces Rise (Lighter):** The base app canvas is dark (bg\_app). Functional panels (like the Cuelist or Active Cues sidebar) sit on top of this canvas and are lighter (bg\_surface).  
2. **Boundaries Recede (Darker):** Panel separation is achieved not by drawn lines, but by showing the bg\_app *between* panels. Borders separating major areas should use bg\_app.  
3. **Interactables Punch Down (Darkest):** Buttons, text inputs, and toggles resting on a bg\_surface do not stick out; they inset into the surface. Their default state (element\_bg) is darker than both the surface they sit on and the base app canvas.

## **Structural Mapping (Referencing image\_4f2717.jpg)**

### **1\. The Environment (App Canvas)**

* **Token:** bg\_app  
* **Usage:** The absolute background of the window. Visible primarily as the 1px dividers *between* major panels (e.g., separating the Main Cuelist from the Active Cues sidebar on the right).

### **2\. Main Panels (Cuelist, Active Cues Sidebar)**

* **Token:** bg\_surface  
* **Usage:** The base background color for the large rectangular areas holding content.  
* **Internal Dividers:** Use border\_divider (lighter than surface) for internal separation, such as the line below the table header.

### **3\. Top Toolbar & Header Area**

* **Container:** Uses bg\_surface.  
* **Big GO Button (Left):** Uses element\_bg (punch-down) with a prominent status\_running border.  
* **Cue Notes Input:** Uses element\_bg (punch-down), element\_border, and text\_primary.  
* **Toolbar Buttons (Add cue, settings, etc.):** Use the Element Interactive tokens (element\_bg, element\_bg\_hover, element\_bg\_active). They appear as dark cutouts in the lighter toolbar surface.  
* **Playback Controls (Right \- Stop, Pause, Play):** These are prominent interactables. Depending on emphasis, they use standard element\_bg or specific semantic backgrounds (e.g., Pause is yellow in the image, Play is dark grey. A modernized Zed approach would keep them element\_bg and rely on iconography or subtle semantic borders unless active).

### **4\. Bottom Status Bar**

* **Container:** Uses bg\_surface.  
* **Content:** Contains layout toggle buttons (Left/Right) and cue counts.  
* **Buttons:** Standard element\_bg punch-down styling. Active state (e.g., showing a panel is open) uses element\_bg\_active.

### **5\. Cuelist Table (Main Area)**

* **Background Pattern:** Alternate rows to improve tracking.  
  * Odd rows: bg\_surface  
  * Even rows: bg\_surface\_raised (Slightly lighter than base surface, *not* a punch-through).  
* **Selection State:** When a row is clicked, it overrides the zebra striping.  
  * Inactive Focus: bg\_selection (Subtle tint).  
  * Active Focus: bg\_selection\_active (Stronger tint).  
* **Playhead / Active Target:** Use status\_playhead for the left-edge indicator (the blue triangle in the image).  
* **Running Cue (Playback State):** When a cue is actively playing (e.g., Cue 11-10 in the image), the row background receives a prominent fill/tint of status\_running (Green). This overrides selection and zebra stripes.  
* **Group Cues:** Group cues (e.g., Cue 12 "walk-in music" in the image) have an outline. Use status\_group (Orange) for the border to denote grouping, drawn atop the row backgrounds.

### **Interaction States**

* **Default:** element\_bg (Darkest, inset).  
* **Hover:** element\_bg\_hover (Slightly lighter, lifting towards the surface).  
* **Pressed/Active:** element\_bg\_active (Deepest inset, near black).  
* **Focus:** Apply a 1px solid border of border\_focus to the outer edge of an element when it receives keyboard focus.