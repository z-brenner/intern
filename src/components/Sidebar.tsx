import { CheckCircle2, CircleAlert, CircleQuestionMark, List, Settings } from 'lucide-react';
import { Icon } from './Icon';
import type { QueueView } from '../types';

const entries = [
  { view: 'queue' as const, label: 'Queue', icon: List },
  { view: 'review' as const, label: 'Needs Review', icon: CircleAlert },
  { view: 'completed' as const, label: 'Completed', icon: CheckCircle2 },
];
export function Sidebar({ active, items, inert, onChange, onSettings, onHelp }: { active: QueueView; items: { status: string }[]; inert?: boolean; onChange(view: QueueView): void; onSettings(trigger: HTMLButtonElement): void; onHelp(): void }) {
  const count = (view: QueueView) => view === 'queue' ? items.filter((item) => item.status !== 'completed').length : items.filter((item) => item.status === (view === 'review' ? 'review' : 'completed')).length;
  return <nav className="sidebar" aria-label="Queue navigation" inert={inert || undefined}>
    {/*
      The count is part of the name: a bare aria-label of "Needs Review"
      replaced the visible "Needs Review 5", so a screen reader never heard how
      much was waiting. Collapsed, the label goes to a tooltip and the count
      rides on the icon, where it used to be hidden along with everything else.
      `data-view` is what code looks a view's button up by, since the name now
      changes with the queue.
    */}
    <div>{entries.map(({ view, label, icon }) => {
      const total = count(view);
      return <button type="button" key={view} data-view={view} className={active === view ? 'nav-item active' : 'nav-item'} onClick={() => onChange(view)} aria-current={active === view ? 'page' : undefined} aria-label={`${label}, ${total}`} title={label}>
        <span className="nav-icon"><Icon icon={icon} /><i className="nav-badge" aria-hidden="true">{total}</i></span><span className="nav-label">{label}</span><b>{total}</b>
      </button>;
    })}</div>
    {/*
      Help sits beside Settings because that is the corner people already
      search when they are stuck, and it opens the published guide in their own
      browser rather than trying to render documentation inside the app.
    */}
    <div className="sidebar-footer">
      <button className="footer-nav" type="button" onClick={onHelp} aria-label="Help & support" title="Help & support"><Icon icon={CircleQuestionMark} /></button>
      <button className="footer-nav" type="button" onClick={(event) => onSettings(event.currentTarget)} aria-label="Settings" title="Settings"><Icon icon={Settings} /></button>
    </div>
  </nav>;
}
