import React, { useState, useEffect } from 'react'
import api from '../../utils/api'
import '../../styles/main.scss'

const SearchBar = ({ searchTerm, setSearchTerm }) => {
  const handleChange = (event) => {
    setSearchTerm(event.target.value)
  }
  return (
    <div className="container-search-bar">
      <input type="text" placeholder="Search by address or ID" value={searchTerm} onChange={handleChange} />
    </div>
  )
}

export default SearchBar
