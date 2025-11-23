import { ArrowUpRight, Clock3, Layers, Search as SearchIcon, ShieldCheck, Sparkles, TerminalSquare } from 'lucide-react';
import { Badge } from './components/ui/badge';
import { Button } from './components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from './components/ui/card';
import { Input } from './components/ui/input';
import { Separator } from './components/ui/separator';
import { ShortcutKey } from './components/ShortcutKey';

type CommandLogEntry = {
  id: string;
  command: string;
  project: string;
  status: 'success' | 'error';
  note: string;
  time: string;
};

const commandLog: CommandLogEntry[] = [
  {
    id: '1',
    command: 'context find --project demo "embeddings"',
    project: 'demo',
    status: 'success',
    note: '5 hits · ranked by recency',
    time: '2m ago'
  },
  {
    id: '2',
    command: 'context put --project context --key web/scaffold --tag ui',
    project: 'context',
    status: 'success',
    note: 'stored 1 document',
    time: '7m ago'
  },
  {
    id: '3',
    command: 'context rm --project demo --id deadbeef',
    project: 'demo',
    status: 'error',
    note: 'document not found',
    time: '12m ago'
  }
];

const shortcuts = [
  { keys: ['/'], label: 'Focus search' },
  { keys: ['?'], label: 'Keyboard help' },
  { keys: ['l'], label: 'Open CLI log' },
  { keys: ['n'], label: 'New document' }
];

const quickActions = [
  {
    title: 'Fresh sync',
    description: 'Pull docs from the CLI SQLite store and tag with the active project.',
    icon: Sparkles
  },
  {
    title: 'Review recents',
    description: 'Skim the latest adds and updates with aging indicators.',
    icon: Clock3
  },
  {
    title: 'Cross-project view',
    description: 'Jump between projects and namespaces without losing focus.',
    icon: Layers
  }
];

export default function App() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="container py-10 space-y-8">
        <header className="glass-panel rounded-3xl border border-border/60 px-6 py-6 md:px-8 shadow-lg shadow-black/40">
          <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
            <div className="space-y-2">
              <div className="flex items-center gap-3 text-sm text-muted-foreground">
                <Badge variant="accent">Context</Badge>
                <span className="pill">React · Vite · shadcn</span>
              </div>
              <div className="space-y-1">
                <h1 className="text-3xl font-semibold leading-tight md:text-4xl">Agent-ready knowledge console</h1>
                <p className="text-base text-muted-foreground md:text-lg">
                  A front-end shell tuned for the Context stack, agent-friendly by default.
                </p>
              </div>
            </div>
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="outline" className="hidden sm:inline-flex">
                Live preview
              </Badge>
              <Button variant="outline" size="lg">
                Open CLI log
              </Button>
              <Button size="lg">
                <Sparkles className="mr-2 h-4 w-4" />
                New document
              </Button>
            </div>
          </div>
          <Separator className="my-6" />
          <div className="grid gap-4 md:grid-cols-[minmax(0,1fr)_240px] md:items-end">
            <div className="space-y-2">
              <label htmlFor="search" className="text-sm text-muted-foreground">
                Search
              </label>
              <div className="relative">
                <SearchIcon className="absolute left-3 top-3 h-5 w-5 text-muted-foreground" />
                <Input
                  id="search"
                  type="search"
                  role="searchbox"
                  placeholder="Search documents, tags, or scenarios..."
                  aria-label="Search"
                  className="pl-11"
                />
              </div>
            </div>
            <div className="space-y-2">
              <label htmlFor="project" className="text-sm text-muted-foreground">
                Project
              </label>
              <div className="relative">
                <select
                  id="project"
                  name="project"
                  aria-label="Project"
                  className="w-full appearance-none rounded-xl border border-border/60 bg-card/80 px-4 py-3 text-sm text-foreground shadow-inner shadow-black/30 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 focus-visible:ring-offset-background"
                  defaultValue="context"
                >
                  <option value="context">context</option>
                  <option value="demo">demo</option>
                  <option value="research">research</option>
                </select>
                <ArrowUpRight className="pointer-events-none absolute right-3 top-3 h-4 w-4 text-muted-foreground" />
              </div>
            </div>
          </div>
        </header>

        <main className="grid gap-6 lg:grid-cols-[1.35fr_1fr]">
          <Card className="rounded-3xl">
            <CardHeader className="pb-4">
              <CardTitle className="flex items-center gap-3 text-xl">
                <ShieldCheck className="h-5 w-5 text-accent" />
                Search-ready workspace
              </CardTitle>
              <CardDescription>Quick controls to explore your documents and prep for UI experiments.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-4 md:grid-cols-3">
                {quickActions.map(({ title, description, icon: Icon }) => (
                  <div
                    key={title}
                    className="rounded-2xl border border-border/50 bg-white/5 p-4 shadow-inner shadow-black/20"
                  >
                    <div className="flex items-center gap-3 text-sm font-semibold text-foreground">
                      <span className="rounded-xl bg-primary/10 p-2 text-primary">
                        <Icon className="h-5 w-5" />
                      </span>
                      {title}
                    </div>
                    <p className="mt-2 text-sm text-muted-foreground">{description}</p>
                  </div>
                ))}
              </div>
              <Separator />
              <div className="grid gap-4 md:grid-cols-2">
                <div className="rounded-2xl border border-border/40 bg-gradient-to-br from-white/8 to-white/2 p-4 shadow-inner shadow-black/30">
                  <div className="flex items-start justify-between gap-2">
                    <div className="space-y-1">
                      <p className="text-sm font-semibold text-foreground">Document health</p>
                      <p className="text-xs text-muted-foreground">Spot stale docs before agents do.</p>
                    </div>
                    <Badge variant="accent">Live</Badge>
                  </div>
                  <div className="mt-4 flex items-center gap-3 text-sm text-muted-foreground">
                    <TerminalSquare className="h-4 w-4 text-primary" />
                    <span>Last sync: 3m ago · 128 docs</span>
                  </div>
                </div>
                <div className="rounded-2xl border border-border/40 bg-gradient-to-br from-primary/10 to-accent/10 p-4 shadow-inner shadow-black/30">
                  <div className="space-y-1">
                    <p className="text-sm font-semibold text-foreground">Saved scenarios</p>
                    <p className="text-xs text-muted-foreground">Pin prompts and walkthroughs as reusable context.</p>
                  </div>
                  <div className="mt-3 flex flex-wrap gap-2 text-xs text-foreground">
                    <span className="pill bg-white/10">qa-runbooks</span>
                    <span className="pill bg-white/10">debug-bundles</span>
                    <span className="pill bg-white/10">docs</span>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          <div className="space-y-4">
            <Card className="rounded-3xl">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center justify-between text-lg">
                  <span>Command log</span>
                  <Badge variant="outline">{commandLog.length} entries</Badge>
                </CardTitle>
                <CardDescription>Recent CLI activity used to populate this workspace.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {commandLog.map((entry) => (
                  <div
                    key={entry.id}
                    data-testid="command-log-entry"
                    className="rounded-2xl border border-border/50 bg-white/5 p-4 shadow-inner shadow-black/20"
                  >
                    <div className="flex items-start justify-between gap-3">
                      <code className="text-sm font-mono text-foreground">{entry.command}</code>
                      <Badge variant={entry.status === 'success' ? 'default' : 'outline'}>
                        {entry.status === 'success' ? 'success' : 'error'}
                      </Badge>
                    </div>
                    <div className="mt-2 flex flex-wrap items-center gap-3 text-xs text-muted-foreground">
                      <span className="rounded-full bg-primary/10 px-2 py-1 text-primary">project: {entry.project}</span>
                      <span className="rounded-full bg-border/60 px-2 py-1">{entry.note}</span>
                      <span className="flex items-center gap-1">
                        <Clock3 className="h-4 w-4" />
                        {entry.time}
                      </span>
                    </div>
                  </div>
                ))}
              </CardContent>
              <CardFooter className="pt-0">
                <Button variant="ghost" className="gap-2 text-sm text-muted-foreground">
                  <ArrowUpRight className="h-4 w-4" />
                  Export log as context
                </Button>
              </CardFooter>
            </Card>

            <Card className="rounded-3xl">
              <CardHeader className="pb-3">
                <CardTitle className="flex items-center gap-2 text-lg">
                  <TerminalSquare className="h-5 w-5 text-primary" />
                  Keyboard shortcuts
                </CardTitle>
                <CardDescription>Optimized for agents and humans in the same loop.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                {shortcuts.map((shortcut) => (
                  <div
                    key={shortcut.label}
                    className="flex items-center justify-between rounded-2xl border border-border/50 bg-white/5 px-4 py-3"
                  >
                    <div className="space-y-0.5">
                      <p className="text-sm font-semibold text-foreground">{shortcut.label}</p>
                      <p className="text-xs text-muted-foreground">
                        {shortcut.label === 'Focus search'
                          ? 'Type, then hit Enter to preview; Escape to reset.'
                          : 'Quick access to high-frequency actions.'}
                      </p>
                    </div>
                    <ShortcutKey keys={shortcut.keys} />
                  </div>
                ))}
              </CardContent>
            </Card>
          </div>
        </main>
      </div>
    </div>
  );
}
