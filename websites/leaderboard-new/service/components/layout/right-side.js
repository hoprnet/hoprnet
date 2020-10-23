import React, { useState, useEffect } from "react";
import CopieParagraph from "../data-view/copie-paragraph";
import DataBoxCloud from "../data-view/data-box-cloud";
import DataUpdateKnow from "../data-view/data-update-know";
import "../../styles/main.scss";

const RightSide = () => {
  return (
    <section className="right-side only-desktop-view">
      <CopieParagraph />
      <hr />
      <DataBoxCloud />
      <hr />
      <DataUpdateKnow />
    </section>
  );
};

export default RightSide;
