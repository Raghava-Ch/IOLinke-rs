Goal
-----
Create a production-ready GUI (renderer) for an **IODD Builder** inside an existing Tauri app. The GUI must be implemented with **Svelte + TypeScript**, styled with **Tailwind CSS**, and use only UI widgets from **flowbite-svelte** for controls (Inputs, Selects, Tabs, Accordion, Modals, Buttons, Navbar, Sidebar, Drawer, Tooltip, Toast, Table, etc.). Use the provided CSV/JSON (iodd_xsd_comprehensive.csv and iodd_form_schema.json) as the authoritative source of UI structure, validation rules, element hierarchy, and business/checker rules. Do not miss or flatten the XML hierarchy — present it visually (tree view / nested panes) and maintain the hierarchy in the generated source files and XML export.

Files available in the repository (source of truth):
- `.temp.details/iodd_xsd_comprehensive.csv`
- existing Tauri app root (the GUI code should be placed into the renderer/UI side of that Tauri project — e.g., `src/renderer/` or `src/` depending on repo conventions)

High-level requirements
-----------------------
1. **Hierarchy-first UI** — Visualize XML hierarchy clearly:
   - Left: collapsible **Hierarchy Tree** (device/profile -> groups -> elements). Use a tree view with icons and right-click context menu (new, duplicate, delete).
   - Center: **Editor area** with **tabs** (one tab per open entity/document). Each tab shows a form for the selected entity using the schema.
   - Right: **Properties / Inspector** panel showing currently selected element metadata, validations, checker rules, cross-reference links, and quick actions.
   - Top: **Activity bar** (like VS Code) with icons for Explorer, Search, Validate, Export, Preview, Settings.
   - Global Navigation bar: breadcrumbs for current selection, device variant selector (if multiple), and global actions (Import CSV, Import IODD XML, Export IODD XML, Run Checker, Save).
   - Layout supports **resizing** (drag to resize panes) and **scrollability** inside panels.

2. **Use flowbite-svelte for all UI elements** — dropdowns, selects, buttons, inputs, toggles, datepickers etc. Do not invent custom UI controls unless flowbite lacks a necessary control; in that case, use minimal Tailwind + accessible markup and note the reason.

3. **Implement 10 representative entities** (complete forms + save/export) — these should cover the main groups and showcase hierarchical handling, multiplicity, attributes, validators, and cross-references. Implement forms for:
   - DocumentInfo
   - ProfileHeader
   - DeviceIdentity
   - DeviceVariantCollection / DeviceVariant
   - DatatypeCollection / Datatype (cover Boolean, Integer, String)
   - VariableCollection / Variable
   - ProcessDataCollection / ProcessData
   - UserInterface / MenuCollection / Menu
   - ExternalTextCollection / Text
   - Stamp

   These 10 entities must be fully functional and demonstrate all required behaviors (add/remove, reorder, validation, multi-language text editing, image upload for deviceIcon/deviceSymbol).

4. **Follow modern UI patterns**:
   - Clean, minimal, responsive layout; consistent spacing; typographic hierarchy.
   - Tabs to separate related sub-groups where appropriate.
   - Group segmentation: group related fields into cards/accordions.
   - Provide keyboard shortcuts (Ctrl/Cmd+S save, Ctrl/Cmd+F search, Ctrl/Cmd+Z undo) and accessible focus states.
   - Provide inline validation errors and a validation summary panel with quick-fix suggestions.
   - Provide a preview mode to show generated IODD XML (pretty-printed) and a downloadable file.

5. **Validation & Checker rules**:
   - Use validators from `/data/iodd_form_schema.json` (enum, pattern, minLength/maxLength, minInclusive/maxInclusive).
   - Enforce `mandatory` fields and show `optional_reason` where relevant.
   - Implement cross-reference checks (e.g., Condition.variableId references an existing Variable; Menu variable refs exist).
   - Show warnings where business/checker rules from `checker_rules` apply (e.g., ProcessData bitLength constraints, global limits).
   - Provide a "Validate" action that runs full spec checks and lists errors and warnings.
   - Add a hook to run the external IODD Checker (shell command) if available; otherwise, provide instructions and a warning.

6. **Data flow & persistence**
   - Read the CSV/JSON schema at startup to drive UI generation (the agent should not hard-code fields except where necessary to bootstrap).
   - Persist changes to the renderer file system state (local JSON file) and provide Export->IODD XML (the exported XML must obey XSD structure).
   - Provide Import->IODD XML to populate the UI (parse and map into forms).
   - Provide Import->CSV to create new DeviceVariant lists or other bulk operations.
   - Use Svelte stores for app state (hierarchy tree, open documents, undo stack). Use TypeScript for types derived from the form schema.

7. **Source code output & structure**
   - Produce a clear file/folder structure under the Tauri renderer, e.g.:
     ```
     src/
       lib/
         schema/             # JSON + helpers to read CSV/form schema
         models/             # TypeScript interfaces auto-generated from schema
         stores/             # Svelte stores
         validators/         # validation helpers
       components/
         ActivityBar.svelte
         HierarchyTree.svelte
         EditorTabs.svelte
         EntityForm.svelte    # dynamic form renderer driven by schema
         InspectorPanel.svelte
         XMLPreview.svelte
         Toolbar.svelte
         Modals/ (Import, Export, Confirm)
       pages/
         Main.svelte
       routes/ (if SvelteKit used)
       styles/
       main.ts
     tauri/ (unchanged)
     ```
   - Provide a **component-per-entity** example for the 10 entities (e.g., `DeviceIdentityForm.svelte`, `VariableForm.svelte`) that use the dynamic `EntityForm` generator where possible but include bespoke logic for complex cases (e.g., Record/Datatype editor, ProcessData condition editor).
   - Provide TypeScript models generated from the CSV/form-schema (automatic codegen file `src/lib/models/generated.ts`).
   - Provide `src/lib/schema/loader.ts` that reads `data/iodd_form_schema.json` at runtime and exposes typed schema.

8. **Form generation & mapping**
   - Implement a dynamic form renderer `EntityForm.svelte` that:
     - Maps `type` → flowbite-svelte widget (enum→Select, string→TextInput or TextArea if long, boolean→Checkbox, integer→NumericInput with min/max, file path→FilePicker).
     - Uses `validators` for real-time validation and shows helpful messages from `help`, `checker_rules`, `optional_reason`.
     - For `multiple` fields, provide array UI (add/remove/reorder) with drag re-order and show max/min counts from `global_limits_references`.
     - For cross-reference fields (variableId, datatypeRef), render a Select populated from current document state; provide quick-create inline (small modal) to add a referenced item.

9. **UX extras**
   - Undo/Redo stack for changes.
   - Drag-and-drop reordering for arrays (DeviceVariant order, Menu entries).
   - Search across fields and elements.
   - Collapsible groups and pinned favorites.
   - Accessibility: ARIA attributes, keyboard navigation, contrast checks.

10. **Testing, linting & docs**
    - Provide unit tests for major components using **Vitest** or preferred test runner, and at least basic tests for:
      - form validation logic,
      - schema loader,
      - XML export mapping.
    - Add lint config (ESLint/TS) and Prettier formatting settings.
    - Create a `README.md` with:
      - how to run the renderer in dev (steps for Tauri),
      - how to import the CSV/JSON,
      - how to run tests,
      - where schema files live,
      - how to add new element mappings.
    - Provide a `DEVELOPER.md` describing codegen steps for regenerating `src/lib/models/generated.ts` from the CSV/JSON.

11. **Commit-ready deliverable**
    - All generated source files must be ready to commit. The agent should create files, not just describe them.
    - Provide small sample data set (example IODD XML) and exported XML from sample edits.

Implementation constraints & guidance
-----------------------------------
- Use **TypeScript** throughout.
- Use **flowbite-svelte** components for all controls. If a control is not available, explain and use Tailwind + accessible markup.
- Keep component code modular and well-documented.
- Ensure exported IODD XML validates against the XSD (to the extent possible in-unit tests).
- Maintain element hierarchy in both UI and saved XML. The UI must reflect nesting and parent/child relationships.
- The agent is allowed to add reasonable UI improvements or missing small features (e.g., loading spinner, toasts) to improve UX — but must not modify or contradict the CSV/spec data.

Deliverables (explicit)
-----------------------
1. A fully-populated `src/` renderer folder with Svelte + TypeScript components and stores (as per structure above).
2. `src/lib/models/generated.ts` — autogenerated types from the CSV/form schema.
3. `src/lib/schema/loader.ts` — runtime loader for `data/iodd_form_schema.json`.
4. `src/components/` — components listed above, with at least `HierarchyTree.svelte`, `EntityForm.svelte`, `InspectorPanel.svelte`, `EditorTabs.svelte`, `ActivityBar.svelte`, `XMLPreview.svelte`.
5. Example per-entity component files for the 10 entities (in `components/forms/`).
6. `data/` sample files: `sample_device.xml`, `sample_state.json`, and ensure `data/iodd_form_schema.json` is read.
7. Tests (Vitest) + basic CI workflow (`.github/workflows/ci.yml`) that runs build and tests.
8. `README.md` and `DEVELOPER.md`.
9. A short `CHANGELOG.md` entry describing implemented features.

Acceptance criteria
-------------------
- The app loads the schema JSON and shows the hierarchy tree for a sample IODD file.
- Opening any of the 10 implemented entities shows a complete form with fields, validators, and cross-reference selects.
- Resizing panels, opening tabs, adding/removing list entries all work.
- Exported XML from a small edited sample validates structurally and preserves hierarchy.
- Validation panel reports both XSD-derived and spec-derived (checker_rules) issues.
- All controls use flowbite-svelte components.

If you need to add reasonable details (file names, additional UI widgets, helper utilities), do that — but **do not change the CSV/spec data**. If the repo contains different renderer paths than assumed, adapt file placement to follow the repo layout and report the final paths in the commit. Add comments at top of every new file explaining purpose and referencing back to the CSV/form schema path.

Start now. Produce a commit-ready patch (list of files created/changed with paths) and include a short README of how to run the dev server in the existing Tauri project. For any decisions made (component names, store structure, third-party libs beyond flowbite-svelte), include a short rationale at the top of the commit message.
