import React from "react";
import { DocsVersion } from "../portals";
import Navbar from "@theme-original/Navbar";

function CustomNavbar() {
  return (
    <>
      <DocsVersion />
      <Navbar />
    </>
  );
}

export default CustomNavbar;
