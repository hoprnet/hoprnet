import React, { useState, useRef } from 'react';
import { usePdf } from '@mikecousins/react-pdf';

const EmbedPdf = ({ src }) => {
  const [page, setPage] = useState(1);
  const canvasRef = useRef(null);

  const { pdfDocument } = usePdf({
    file: src,
    page,
    canvasRef,
    scale: 2
  });

  return (
    <div style={{
      display: "flex",
      justifyContent: "flex-start",
      width: "100vw",
      alignItems: "center",
      flexDirection: "column",
      marginTop: "50px",
    }}>
      {!pdfDocument && <span>Loading...</span>}
      <canvas ref={canvasRef} width="842" height="595" />
      {Boolean(pdfDocument && pdfDocument.numPages) && (
        <nav>
          <ul className="pager">
            <li className="previous">
              <button disabled={page === 1} onClick={() => setPage(page - 1)}>
                Previous
              </button>
            </li>
            <li className="next">
              <button
                disabled={page === pdfDocument.numPages}
                onClick={() => setPage(page + 1)}
              >
                Next
              </button>
            </li>
          </ul>
        </nav>
      )}
    </div>
  );
}

export default EmbedPdf
