// Source code display panel with syntax highlighting

interface CodeTab {
  label: string;
  code: string;
  language: 'rust' | 'typescript';
}

export function createCodePanel(tabs: CodeTab[]): {
  container: HTMLElement;
  toggle: HTMLElement;
} {
  let isOpen = false;
  let activeTab = 0;

  // Toggle button (positioned in canvas area on desktop)
  const toggle = document.createElement('div');
  toggle.className = 'code-panel-toggle';
  toggle.textContent = 'View Source Code';

  // Mobile toggle button (shown inline before code panel on mobile)
  const mobileToggle = document.createElement('div');
  mobileToggle.className = 'code-panel-toggle code-panel-toggle-mobile';
  mobileToggle.textContent = 'View Source Code';

  // Panel container
  const container = document.createElement('div');
  container.className = 'code-panel';

  // Mobile toggle sits inside container but outside the collapsible area
  container.appendChild(mobileToggle);

  // Collapsible inner wrapper (this is what expands/collapses)
  const collapsible = document.createElement('div');
  collapsible.className = 'code-panel-collapsible';

  // Header
  const header = document.createElement('div');
  header.className = 'code-header';

  const tabBar = document.createElement('div');
  tabBar.className = 'code-tabs';

  const copyBtn = document.createElement('button');
  copyBtn.className = 'copy-btn';
  copyBtn.textContent = 'Copy';
  copyBtn.addEventListener('click', () => {
    navigator.clipboard.writeText(tabs[activeTab].code);
    copyBtn.textContent = 'Copied!';
    setTimeout(() => copyBtn.textContent = 'Copy', 1500);
  });

  header.appendChild(tabBar);
  header.appendChild(copyBtn);

  // Content
  const content = document.createElement('div');
  content.className = 'code-content';
  const pre = document.createElement('pre');
  content.appendChild(pre);

  collapsible.appendChild(header);
  collapsible.appendChild(content);
  container.appendChild(collapsible);

  function renderTabs() {
    tabBar.innerHTML = '';
    tabs.forEach((tab, i) => {
      const btn = document.createElement('button');
      btn.className = 'code-tab' + (i === activeTab ? ' active' : '');
      btn.textContent = tab.label;
      btn.addEventListener('click', () => {
        activeTab = i;
        renderTabs();
        renderCode();
      });
      tabBar.appendChild(btn);
    });
  }

  function renderCode() {
    const tab = tabs[activeTab];
    pre.innerHTML = highlightSyntax(tab.code, tab.language);
  }

  function toggleOpen() {
    isOpen = !isOpen;
    collapsible.classList.toggle('open', isOpen);
    const label = isOpen ? 'Hide Source Code' : 'View Source Code';
    toggle.textContent = label;
    mobileToggle.textContent = label;
  }

  toggle.addEventListener('click', toggleOpen);
  mobileToggle.addEventListener('click', toggleOpen);

  renderTabs();
  renderCode();

  return { container, toggle };
}

function highlightSyntax(code: string, lang: 'rust' | 'typescript'): string {
  // Escape HTML first
  let html = code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  if (lang === 'rust') {
    // Comments
    html = html.replace(/(\/\/.*$)/gm, '<span class="cmt">$1</span>');
    // Strings
    html = html.replace(/("(?:[^"\\]|\\.)*")/g, '<span class="str">$1</span>');
    // Macros
    html = html.replace(/\b(\w+!)\(/g, '<span class="mac">$1</span>(');
    // Keywords
    html = html.replace(/\b(fn|let|mut|pub|use|struct|enum|impl|match|if|else|for|while|loop|return|const|self|true|false|as|in|ref|where|type|trait|mod|crate|super|unsafe|async|await|move|dyn|static|extern)\b/g, '<span class="kw">$1</span>');
    // Types
    html = html.replace(/\b(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|usize|isize|bool|str|String|Vec|Option|Result|Path64|Paths64|Point64|Rect64|FillRule|ClipType|JoinType|EndType|PolyTree64)\b/g, '<span class="ty">$1</span>');
    // Numbers
    html = html.replace(/\b(\d+\.?\d*(?:f64|f32|i64|i32|u32|usize)?)\b/g, '<span class="num">$1</span>');
  } else {
    // TS/JS
    html = html.replace(/(\/\/.*$)/gm, '<span class="cmt">$1</span>');
    html = html.replace(/("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)/g, '<span class="str">$1</span>');
    html = html.replace(/\b(function|const|let|var|return|if|else|for|while|import|export|from|async|await|new|typeof|class|extends|true|false|null|undefined)\b/g, '<span class="kw">$1</span>');
    html = html.replace(/\b(number|string|boolean|void|any|Float64Array)\b/g, '<span class="ty">$1</span>');
    html = html.replace(/\b(\d+\.?\d*)\b/g, '<span class="num">$1</span>');
  }

  return html;
}
