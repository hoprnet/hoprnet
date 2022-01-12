/*
  This component adds links to docs releases in the navbar dropdown.
  Unfortunately, Docosaurus doesn't support custom react components in the navbar. That's why the react [portal](https://reactjs.org/docs/portals.html) is used to add docs versions to the dropdown list.
  Releases tags are fetched from github API, and converted to docs versions.
*/
import React, { useEffect, useState } from 'react'
import ReactDOM from 'react-dom'
import { CHOOSE_DOCS_VERSION_ID, LATEST_VERSION_NAME, DOCS_URL } from '../../consts'
import { github } from '../api'

const ListItem = ({ name, link }) => (
  <li>
    <a href={`${DOCS_URL}/${link}`} className="dropdown__link">
      <span>{name}</span>
    </a>
  </li>
)

const List = ({ versions, isLoading, error }) => {
  if (isLoading) {
    return <div>Loading...</div>
  }

  if (error) {
    return <div>{error}</div>
  }

  return (
    <>
      {versions.map(({ name, link }) => (
        <ListItem key={name} name={name} link={link} />
      ))}
    </>
  )
}

export default function DocsVersion() {
  const [versions, setVersions] = useState([])

  const [list, setList] = useState(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    setList(document.getElementById(CHOOSE_DOCS_VERSION_ID)?.nextElementSibling || null)
  }, [setList])

  useEffect(() => {
    github
      .getReleases()
      .then((data) => {
        setVersions([
          { name: LATEST_VERSION_NAME, link: LATEST_VERSION_NAME },
          ...data.map(({ tag_name }) => ({
            name: tag_name,
            link: tag_name
          }))
        ])

        setIsLoading(false)
      })
      .catch((err) => {
        setError(err.message)
        setIsLoading(false)
      })
  }, [setVersions])

  if (list === null) {
    return null
  }

  return ReactDOM.createPortal(<List isLoading={isLoading} versions={versions} error={error} />, list)
}
