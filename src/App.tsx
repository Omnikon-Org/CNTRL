import { onMount } from 'solid-js';
import { browserState, browserActions } from './stores/browserStore';
import { initAiStore } from './stores/aiStore';
import { TabBar } from './components/TabBar';
import { UrlBar } from './components/UrlBar';
import { WebView } from './components/WebView';
import './App.css';

function App() {
  onMount(async () => {
    await initAiStore();
    await browserActions.fetchTabs();
    if (browserState.tabs.length === 0) {
      await browserActions.openTab('https://google.com');
    }
  });

  return (
    <div class="app-container">
      <TabBar />
      <UrlBar />
      <WebView />
    </div>
  );
}

export default App;
