import React, { useState, useEffect } from "react";
import "../../styles/main.scss";

const SearchBar = ({ searchTerm, setSearchTerm, match }) => {
  const handleChange = (event) => {
    setSearchTerm(event.target.value);
  };
  return (
    <div className="container-search-bar">
      <p className="help-total-results">
        Total results: <span>{match}</span>
      </p>
      <input
        className="search"
        type="text"
        placeholder="Search by address or ID"
        value={searchTerm}
        onChange={handleChange}
      />
    </div>
  );
};

export default SearchBar;
