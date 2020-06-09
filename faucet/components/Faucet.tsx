import { useState, useEffect } from 'react'
import { isAddress } from 'web3-utils'
import type { addresses } from '@hoprnet/hopr-ethereum'

async function mint(
  network: addresses.Networks,
  address: string
): Promise<{
  success: boolean
  transactionHash?: string
  message?: string
}> {
  try {
    const response = await fetch(`/api/mint?network=${network}&address=${address}`)

    if (response.status === 200) {
      const { transactionHash } = await response.json()
      return {
        success: true,
        transactionHash,
      }
    } else {
      return {
        success: false,
        message: response.statusText,
      }
    }
  } catch (err) {
    console.error(err)

    return {
      success: false,
      message: 'Unexpected error',
    }
  }
}

function Faucet({ network }: { network: addresses.Networks }) {
  // initializing states as 'undefined' to signify that state is uninitialized
  const [address, setAddress] = useState<string>(undefined)
  const [isValidAddress, setIsValidAddress] = useState<boolean>(undefined)
  const [status, setStatus] = useState<'PENDING' | 'SUCCESS' | 'FAILURE'>(undefined)
  const [message, setMessage] = useState<string>(undefined)
  const isButtonDisabled = !isValidAddress || status === 'PENDING'

  // update 'isValidAddress' on 'address' change
  useEffect(() => {
    // check if address is initialized or empty (when a user hits backspace)
    if (typeof address === 'undefined' || address === '') {
      setIsValidAddress(undefined)
    } else {
      setIsValidAddress(isAddress(address))
    }
  }, [address])

  // once button is clicked
  const onClick = async (network: addresses.Networks, address: string): Promise<void> => {
    try {
      setStatus('PENDING')

      const result = await mint(network, address)

      if (result.success) {
        setMessage(result.transactionHash)
        setStatus('SUCCESS')
      } else {
        setMessage(result.message)
        setStatus('FAILURE')
      }
    } catch (err) {
      console.error(err)

      setMessage('Unexpected error')
      setStatus('FAILURE')
    }
  }

  return (
    <div className="container">
      <div className="inputs">
        <input onChange={(e) => setAddress(e.target.value)} defaultValue={address} />
        &nbsp;
        <button onClick={() => onClick(network, address)} disabled={isButtonDisabled}>
          Give me hopr tokens!
        </button>
      </div>
      {status === 'SUCCESS' ? (
        <div>
          ✔ Your transaction is pending:{' '}
          <a href={`http://${network}.etherscan.io/tx/${message}`} target="_blank" rel="noopener noreferrer">
            etherscan
          </a>
        </div>
      ) : status === 'FAILURE' ? (
        <div>
          <p>🤕 Something broke: '{message}'</p>
        </div>
      ) : null}
      <style jsx>{`
        .container {
          display: flex;
          flex-direction: column;
          justify-content: space-between;
          align-items: center;
          padding: 0.5 0rem;
          height: 100px;
        }

        .inputs {
          display: flex;
          flex-direction: row;
          justify-content: center;
          align-items: center;
          padding: 0 0.5rem;
        }

        input {
          width: 310px;
          max-width: 100%;
          line-height: 2.5em;
          border: 1px solid ${isValidAddress === false ? 'red' : 'black'};
        }

        button {
          background-color: #3c4146;
          color: #eef4ec;
          padding: 0.5em;
          border: none;
          width: 150px;
          cursor: ${isButtonDisabled ? 'not-allowed' : 'pointer'};
          opacity: ${isButtonDisabled ? 0.75 : 1};
        }
      `}</style>
    </div>
  )
}

export default Faucet
