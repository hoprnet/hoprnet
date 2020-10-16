import HOPRCountdown from '../components/Countdown/Countdown'
import Logo from '../components/Logo/Logo'


function HomeContent() {
  return (
    <>
      <Logo />
      <div>
        <HOPRCountdown />
      </div>
      <small style={{ display: 'block', textAlign: 'center', fontSize: '.8em', margin: '10px auto'}}>© 2020 HOPR Association, all rights reserved</small>
    </>
  )
}


const Home: React.FC = () => {
  return <HomeContent />
}

export default Home
