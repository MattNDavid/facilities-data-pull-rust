import EventsList from './EventsList'
import SetupViewer from './SetupViewer'
import './App.css'

function App() {
  const params = new URLSearchParams(window.location.search)
  const roomSetupId = params.get('setup')

  if (roomSetupId) {
    return <SetupViewer roomSetupId={roomSetupId} />
  }

  return <EventsList />
}

export default App
