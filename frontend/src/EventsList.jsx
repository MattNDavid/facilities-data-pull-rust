import { useState, useEffect } from 'react'

const API_BASE = '/api'

function formatTime(timeStr) {
  const [h, m] = timeStr.split(':')
  const hour = parseInt(h, 10)
  const ampm = hour >= 12 ? 'PM' : 'AM'
  const displayHour = hour % 12 || 12
  return `${displayHour}:${m} ${ampm}`
}

function formatDate(dateStr) {
  const [year, month, day] = dateStr.split('-').map(Number)
  return new Date(year, month - 1, day).toLocaleDateString('en-US', {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
  })
}

function ScheduleSublist({ eventName }) {
  const [rows, setRows] = useState(null)
  const [error, setError] = useState(null)

  useEffect(() => {
    const params = new URLSearchParams({ event_name: eventName })
    fetch(`${API_BASE}/schedule?${params}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`)
        return r.json()
      })
      .then(setRows)
      .catch(err => setError(err.message))
  }, [eventName])

  if (error) return <p className="status error">Error loading schedule: {error}</p>
  if (!rows) return <p className="status sublist-loading">Loading...</p>

  if (rows.length === 0) return <p className="status">No schedule entries found.</p>

  return (
    <ul className="schedule-sublist">
      {rows.map(row => (
        <li key={row.id} className="schedule-row">
          <div className="schedule-row-main">
            <span className="schedule-resource">{row.resource_name}</span>
            <span className="schedule-time">
              {formatTime(row.start_time)} – {formatTime(row.end_time)}
            </span>
            {row.room_setup_id && (
              <a
                className="setup-link"
                href={`/?setup=${row.room_setup_id}`}
                target="_blank"
                rel="noreferrer"
              >
                View setup
              </a>
            )}
          </div>
          {(row.first_name || row.last_name || row.email) && (
            <div className="schedule-contact">
              {[row.first_name, row.last_name].filter(Boolean).join(' ')}
              {row.email && <span className="schedule-email"> — {row.email}</span>}
            </div>
          )}
          {row.notes && <div className="schedule-notes">{row.notes}</div>}
          {row.question && row.answer && (
            <div className="schedule-qa">
              <span className="schedule-question">{row.question}:</span> {row.answer}
            </div>
          )}
        </li>
      ))}
    </ul>
  )
}

export default function EventsList() {
  const [events, setEvents] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [dateGte, setDateGte] = useState('')
  const [dateLte, setDateLte] = useState('')
  const [expandedId, setExpandedId] = useState(null)

  useEffect(() => {
    const params = new URLSearchParams()
    if (dateGte) params.set('date_gte', dateGte)
    if (dateLte) params.set('date_lte', dateLte)

    setLoading(true)
    setError(null)
    setExpandedId(null)
    fetch(`${API_BASE}/events?${params}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`)
        return r.json()
      })
      .then(data => { setEvents(data); setLoading(false) })
      .catch(err => { setError(err.message); setLoading(false) })
  }, [dateGte, dateLte])

  const grouped = events.reduce((acc, event) => {
    const key = event.date
    if (!acc[key]) acc[key] = []
    acc[key].push(event)
    return acc
  }, {})

  function toggleExpand(id) {
    setExpandedId(prev => (prev === id ? null : id))
  }

  return (
    <div className="events-list">
      <h1>Events Schedule</h1>

      <div className="filters">
        <label>
          From
          <input type="date" value={dateGte} onChange={e => setDateGte(e.target.value)} />
        </label>
        <label>
          To
          <input type="date" value={dateLte} onChange={e => setDateLte(e.target.value)} />
        </label>
        {(dateGte || dateLte) && (
          <button onClick={() => { setDateGte(''); setDateLte('') }}>Clear</button>
        )}
      </div>

      {loading && <p className="status">Loading...</p>}
      {error && <p className="status error">Error: {error}</p>}
      {!loading && !error && events.length === 0 && (
        <p className="status">No events found.</p>
      )}

      {Object.keys(grouped).sort().map(date => (
        <div key={date} className="day-group">
          <h2>{formatDate(date)}</h2>
          <ul>
            {grouped[date].map(event => {
              const isExpanded = expandedId === event.event_instance_id
              return (
                <li
                  key={event.event_instance_id}
                  className={`event-item${isExpanded ? ' expanded' : ''}`}
                >
                  <button
                    className="event-item-header"
                    onClick={() => toggleExpand(event.event_instance_id)}
                    aria-expanded={isExpanded}
                  >
                    <span className="event-time">{formatTime(event.start_time)}</span>
                    <span className="event-name">{event.event_name}</span>
                    <span className="expand-chevron">{isExpanded ? '▲' : '▼'}</span>
                  </button>
                  {isExpanded && (
                    <ScheduleSublist eventName={event.event_name} />
                  )}
                </li>
              )
            })}
          </ul>
        </div>
      ))}
    </div>
  )
}
