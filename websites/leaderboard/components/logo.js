import styles from '../styles/logo.module.css'

export default function Logo(props) {
  return (
    <div className={styles.logo} onClick={props.onClick}>
      <svg
        version="1.1"
        xmlns="http://www.w3.org/2000/svg"
        xmlnsXlink="http://www.w3.org/1999/xlink"
        x="0px"
        y="0px"
        viewBox="0 0 196 196"
        style={{ enableBackground: 'new 0 0 196 196' }}
        xmlSpace="preserve"
      >
        <circle cx="98" cy="98" r="98" fill="#FFFFA0" />
        <linearGradient id="G" gradientUnits="userSpaceOnUse" x1="102" y1="163.0232" x2="102" y2="57.4945">
          <stop offset="0" style={{ stopColor: '#0000B4' }} />
          <stop offset="9.090910e-03" style={{ stopColor: '#0000B4' }} />
          <stop offset="1" style={{ stopColor: '#000050' }} />
        </linearGradient>
        <path
          fill="url(#G)"
          d="M115.9,61.7c-1.1-0.1-2.2-0.2-3.3-0.2c-8.2,0-16.3,3.3-23.5,9.8c-3.4,3-6.5,6.6-9.4,10.8
  c-0.5-15.6-1.4-32.2-2.9-49.1H55.6c4.5,47.7,4,89.3,3.2,125.8h0c0,0.1,0,0.1,0,0.2h21c0.6-36.6,9.8-60,23.3-72.1
  c3.6-3.3,7.2-4.7,10.5-4.4c16.6,1.8,14,53.6,12.7,76.5h21c0.9-16.9,1.9-37.5-0.5-55.5C143.3,77.6,132.9,63.6,115.9,61.7z"
        />
      </svg>
    </div>
  )
}
