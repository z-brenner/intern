/*
  A press acts only on a control the person can have aimed it at.

  A press is aimed at what is under the pointer as the hand sets out, and
  lands a moment later on whatever is there by then. The window changes by
  itself in between: a document is read again, or leaves the queue, and
  what was typed for it is kept in a strip above the fields. With real
  presses a quarter of a second after such a change, a press on its way to
  Close review landed on Rename all ready and renamed three documents; one
  on its way to the name landed on "Discard what I had", and what had been
  typed was gone; one on its way to Keep original landed on Approve & rename
  and filed a reading nobody had looked at; and the second click of a
  double-click on Discard landed on Cancel processing.

  So each control that acts - one that decides, files, removes, discards,
  cancels, retries or renames - is marked where it is drawn (`data-acts`),
  and a press of the pointer on one is taken only when that control has been
  under the place pressed for the settle time. Where it has only just come
  there - it moved, it appeared, or what covered it went - the press does
  nothing: the page hears of no button going down and of no click, the
  focus and any selection stay as they were, and nothing is shown. The
  press is to be made again.

  One kind of control is taken sooner: one that came where it is in answer
  to the person's own press or key - an item of a menu just opened, the
  buttons of a panel just asked for - once the pointer has gone to it from
  where it was then. That travel is the aim. Pressed where the pointer
  already was, it is not taken: the second click of a double-click lands
  there. A change that came by itself is never taken sooner, wherever the
  pointer has been since: a hand already on its way moves too.

  And two buttons are the same two for every document: Approve & rename and
  Keep original (`data-acts="approve"`, `"keep"`). While a decision of the
  person's is carried out they are disabled, and the panel shows the
  document just decided as filed - without them - for a moment before it
  shows the next. One of them that went while it was disabled, and is back
  in the same place within the settle time, has not arrived there: it is
  where the person left it, and approving one document after another with
  the pointer at rest goes on. One that went while it could be pressed - its
  document read again by itself - has arrived when it comes back.

  Keys are not looked at here. A click that comes from the keyboard (Enter
  or the space bar on a button) carries no press of the pointer and passes;
  what keys may do while the window changes is review's own matter
  (TypedKept, typingInReview).
*/

/**
 * How long a control must have been under the place pressed, in
 * milliseconds. Half a second: Windows' double-click time unless a person
 * has set another, so at that setting the second click of a double-click is
 * not taken on what the first brought under it; and twice the quarter of a
 * second after a change at which the presses above were made. It is also
 * what a deliberate press costs when it is made sooner than that after the
 * control arrived: it has to be made again.
 */
export const SETTLE_MS = 500;

/**
 * How far from where the pointer was, in CSS pixels, a press must land to
 * count as the pointer having gone to a control the person asked for. A
 * chosen number. How far a hand strays between the two clicks of a
 * double-click, or a finger between two taps on one spot, was not measured.
 */
export const REACH_PX = 24;

/** A control's box in the window, as `getBoundingClientRect` gives it. */
export interface Box { left: number; top: number; right: number; bottom: number }

interface Point { x: number; y: number }

/**
 * Where a control was from `at` on. No `box`: it was not there to be
 * pressed - not drawn, or its middle covered. `own`: it came there in
 * answer to the person's own press or key, with the pointer then at
 * `pointer` (undefined before any pointer has been seen). `left`: it went
 * from the page then, in the middle of a decision of the person's own.
 */
interface Place { at: number; box?: Box; own: boolean; pointer?: Point; left?: boolean }

/**
 * A control that acts, as the page shows it now: its box, and for one of
 * the two that are the same for every document, whether a decision of the
 * person's is being carried out (`deciding`: it is disabled).
 */
export type Seen = readonly [control: object, box: Box | undefined, deciding?: boolean];

const within = (box: Box | undefined, point: Point) => box !== undefined && point.x >= box.left && point.x < box.right && point.y >= box.top && point.y < box.bottom;
const same = (one: Box | undefined, other: Box | undefined) => one === other
  || (one !== undefined && other !== undefined && one.left === other.left && one.top === other.top && one.right === other.right && one.bottom === other.bottom);

/**
 * The rule itself, with no page and no clock of its own: it is told where
 * the controls are each time that may have changed (`seen`), where the
 * pointer is (`pointerAt`), and answers whether a press is refused.
 */
export function createAim(settle = SETTLE_MS) {
  // Each control's places, oldest first; the last is where it is now. Only
  // those of the last `settle` milliseconds are kept, and what came before
  // them as far as a press can still be judged by it.
  const known = new WeakMap<object, { places: Place[]; deciding: boolean }>();
  let there = new Set<object>();
  let pointer: Point | undefined;
  /** Whether the place at `index` is a going in the middle of the person's own decision, back within the settle time. */
  const leftForAMoment = (places: Place[], index: number) => places[index].left === true && index + 1 < places.length && places[index + 1].at - places[index].at < settle;
  return {
    /** The pointer was seen here: a move, a press or a release. */
    pointerAt(x: number, y: number) { pointer = { x, y }; },
    /**
     * Every control that acts, as of `now`. `own`: this is looked at while
     * the person's own press or key is being handled, so what has changed
     * since the last look, they changed.
     */
    seen(controls: Iterable<Seen>, now: number, own = false) {
      const present = new Set<object>();
      for (const [control, box, deciding = false] of controls) {
        present.add(control);
        const record = known.get(control);
        if (!record) { known.set(control, { places: [{ at: now, box, own, pointer }], deciding }); continue; }
        const { places } = record;
        record.deciding = deciding;
        if (!same(places[places.length - 1].box, box)) places.push({ at: now, box, own, pointer });
        // Nothing older than the place it was in a settle time ago is asked
        // about again - but the place before a going that may yet be passed over.
        let old = 0;
        while (old + 1 < places.length && places[old + 1].at <= now - settle) old += 1;
        if (old > 0 && places[old].left) old -= 1;
        places.splice(0, old);
      }
      // Gone from the page. Back later, it has appeared again.
      for (const control of there) {
        const record = present.has(control) ? undefined : known.get(control);
        if (record) record.places.push({ at: now, own, pointer, left: record.deciding });
      }
      there = present;
    },
    /** Whether a press at (x, y) on `control` at `now` is refused. */
    refuses(control: object, x: number, y: number, now: number): boolean {
      const places = known.get(control)?.places;
      // Never seen where it is: it has only now appeared.
      if (!places) return true;
      const pressed = { x, y };
      // When it came under the place pressed, to stay: the first of the
      // unbroken run of places, up to now, that have that place in them. A
      // control pressed where its box does not say it is - at an edge left
      // showing while its middle is covered - is judged by when it last changed.
      let arrived = places.length - 1;
      if (within(places[arrived].box, pressed)) {
        for (;;) {
          if (arrived > 0 && within(places[arrived - 1].box, pressed)) arrived -= 1;
          // Back where the person left it (see above): the run goes on from before it went.
          else if (arrived > 1 && leftForAMoment(places, arrived - 1) && within(places[arrived - 2].box, pressed)) arrived -= 2;
          else break;
        }
      }
      const arrival = places[arrived];
      if (now - arrival.at >= settle) return false;
      // Too soon to have been seen there. Taken all the same when the person
      // asked for it and the pointer has gone to it since.
      return !(arrival.own && arrival.pointer !== undefined && Math.hypot(x - arrival.pointer.x, y - arrival.pointer.y) >= REACH_PX);
    },
  };
}

/** What marks a control that acts, where it is drawn. */
const ACTS = '[data-acts]';

/**
 * A control's box while it is there to be pressed: drawn, and on top at its
 * middle. Under the drawer, behind Settings, scrolled under the panel's
 * decision or out of sight below it, it has none - and has only just come
 * where it is when what covered it goes.
 */
function boxOf(control: Element): Box | undefined {
  const { left, top, right, bottom } = control.getBoundingClientRect();
  if (right <= left || bottom <= top) return undefined;
  const page = control.ownerDocument;
  // A page that cannot say what is on top (jsdom) leaves the box to speak for itself.
  if (typeof page.elementFromPoint !== 'function') return { left, top, right, bottom };
  const onTop = page.elementFromPoint((left + right) / 2, (top + bottom) / 2);
  return onTop !== null && control.contains(onTop) ? { left, top, right, bottom } : undefined;
}

/**
 * The controls that act on a page, each as the rule is to know it. One
 * marked with a name (`data-acts="approve"`) is the same control whichever
 * button stands for it at the moment - the panel draws a new one for each
 * document - so it is known by the name; any other is known by itself.
 */
function acting(page: Document) {
  const named = new Map<string, object>();
  const whose = new Map<Element, object>();
  return {
    /** As the page is now. */
    now(): Seen[] {
      whose.clear();
      const taken = new Set<object>();
      return [...page.querySelectorAll(ACTS)].map((control) => {
        const name = control.getAttribute('data-acts');
        let one: object = control;
        if (name && name !== 'true') {
          const standing = named.get(name) ?? { name };
          named.set(name, standing);
          // Only one button on the page stands for a name; a second is itself.
          if (!taken.has(standing)) one = standing;
        }
        taken.add(one);
        whose.set(control, one);
        return [one, boxOf(control), one !== control && (control as HTMLButtonElement).disabled === true] as const;
      });
    },
    /** What `control` was known as when the page was last looked at. */
    known: (control: Element): object => whose.get(control) ?? control,
  };
}

/**
 * Applies the rule to a window: from here on, a press of the pointer on a
 * control that acts (`data-acts`) is taken only when it can have been aimed
 * at it (see above). Returns how to stop.
 *
 * Where the controls are is looked at whenever the page changes (a
 * MutationObserver), scrolls or is resized, when a field takes a character
 * (a name grows by a line with no change to the page), and again as each
 * press lands - so a change none of these saw counts from the press that
 * found it, and that press is the one refused.
 */
export function takeOnlyAimedPresses(view: Window = window, clock: () => number = () => view.performance.now()): () => void {
  const page = view.document;
  const aim = createAim();
  // While a press or a key of the person's own is being handled: what
  // changes on the page then, they changed.
  let own = false;
  let ownEnds: number | undefined;
  // The press that is down, or was last: refused, and so is all that follows from it.
  let refused = false;
  const controls = acting(page);
  const look = () => aim.seen(controls.now(), clock(), own);
  const observer = new MutationObserver(look);
  const theirs = () => {
    own = true;
    // Until the event has been through the page (`through`); for one that
    // is stopped on its way, until the browser turns to what comes next.
    view.clearTimeout(ownEnds);
    ownEnds = view.setTimeout(() => { own = false; }, 0);
  };
  // What the handlers changed and the observer has not been told of yet is
  // seen while it is still the person's. (A browser tells the observer
  // between one listener and the next; an event sent from code does not.)
  const through = () => { if (own && observer.takeRecords().length > 0) look(); own = false; };
  const drop = (event: Event) => { event.preventDefault(); event.stopImmediatePropagation(); };
  /** Whether the press this event begins, or ends, lands on a control it cannot have been aimed at. */
  const misses = (event: MouseEvent) => {
    // What changed before this was not its doing.
    own = false;
    look();
    const control = event.target instanceof Element ? event.target.closest(ACTS) : null;
    return control !== null && aim.refuses(controls.known(control), event.clientX, event.clientY, clock());
  };
  const pointerDown = (event: PointerEvent) => {
    // Only the first pointer down clicks; a second finger changes nothing here.
    if (!event.isPrimary) return;
    // Whichever button: the right one clicks nothing, but its press takes
    // the focus too, and Enter or the space bar would act from there.
    refused = misses(event);
    aim.pointerAt(event.clientX, event.clientY);
    // Refused here, the browser sends no mousedown: the focus stays where it
    // was and no selection starts.
    if (refused) drop(event); else theirs();
  };
  const mouseDown = (event: MouseEvent) => {
    // A button pressed while another is held has no pointerdown of its own.
    if ((event.buttons & (event.buttons - 1)) !== 0 && !refused) refused = misses(event);
    if (refused) drop(event); else theirs();
  };
  // The button let go again reaches the page whatever became of its press:
  // nothing acts on it, and the table ends a drag by it.
  const pointerUp = (event: PointerEvent) => {
    aim.pointerAt(event.clientX, event.clientY);
    if (!refused) theirs();
  };
  const mouseUp = () => { if (!refused) theirs(); };
  const click = (event: MouseEvent) => {
    // From the keyboard, or from code: no press of the pointer.
    if (event.detail === 0) { theirs(); return; }
    // Judged again where it lands: a finger's click goes to what is under
    // it as it lifts, which the press itself can have uncovered.
    if (refused || misses(event)) drop(event); else theirs();
  };
  const doubleClick = (event: MouseEvent) => { if (refused) drop(event); else theirs(); };
  const moved = (event: PointerEvent) => {
    aim.pointerAt(event.clientX, event.clientY);
    // A button pressed while another is held comes as a move that names the
    // button (and a mousedown, unless the press before it was refused): it
    // is judged here, where refusing it keeps the focus where it was.
    if (!event.isPrimary || event.button < 0 || (event.buttons & [1, 4, 2, 8, 16][event.button]) === 0) return;
    if (!refused) refused = misses(event);
    if (refused) drop(event);
  };
  const typed = () => { theirs(); look(); };
  observer.observe(page.documentElement, { subtree: true, childList: true, attributes: true, characterData: true });
  const capture: Array<[EventTarget, string, (event: never) => void]> = [
    [view, 'pointerdown', pointerDown], [view, 'mousedown', mouseDown], [view, 'pointerup', pointerUp], [view, 'mouseup', mouseUp],
    [view, 'click', click], [view, 'dblclick', doubleClick], [view, 'pointermove', moved], [view, 'keydown', theirs], [view, 'keyup', theirs], [view, 'input', typed],
    [page, 'scroll', look], [view, 'resize', look],
  ];
  const bubbled = ['pointerdown', 'mousedown', 'pointerup', 'mouseup', 'click', 'dblclick', 'keydown', 'keyup', 'input'];
  for (const [target, type, listener] of capture) target.addEventListener(type, listener as EventListener, true);
  for (const type of bubbled) view.addEventListener(type, through);
  look();
  return () => {
    observer.disconnect();
    view.clearTimeout(ownEnds);
    for (const [target, type, listener] of capture) target.removeEventListener(type, listener as EventListener, true);
    for (const type of bubbled) view.removeEventListener(type, through);
  };
}
