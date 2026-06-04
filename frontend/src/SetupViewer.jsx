import { useState, useEffect } from 'react'

const API_BASE = '/api'

export default function SetupViewer({ roomSetupId }) {
  const [setup, setSetup] = useState(null)
  const [error, setError] = useState(null)

  useEffect(() => {
    fetch(`${API_BASE}/setup?room_setup_id=${roomSetupId}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`)
        return r.json()
      })
      .then(setSetup)
      .catch(err => setError(err.message))
  }, [roomSetupId])

  if (error) return <p className="status error">Error: {error}</p>
  if (!setup) return <p className="status">Loading setup...</p>

  return (
    <div className="setup-viewer">
      <h1>{setup.name}</h1>
      {setup.url
        ? <img src={setup.url} alt={`${setup.name} diagram`} className="setup-diagram" />
        : <p className="status">No diagram available for this setup.</p>
      }
    </div>
  )
}
