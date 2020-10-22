import React, { useState, useEffect } from 'react'
import api from '../../utils/api'
import '../../styles/main.scss'

const SearchBar = ({ searchTerm, setSearchTerm, getForSearchBar }) => {
  const handleChange = (event) => {
    setSearchTerm(event.target.value)
  }
  return (
    <div className="container-search-bar">
      <input type="text" placeholder="Search" value={searchTerm} onChange={handleChange} />
      <div
        className="btn-search"
        onClick={() => {
          getForSearchBar()
        }}
      >
        <img src="/assets/icons/magnifying.svg" alt="magnifying search-bar" />
      </div>
    </div>
  )
}

export default SearchBar
