import {useCallback, useState} from "react";
import "./App.css";
import "./JSONEditorReact"
import JSONEditorReact from "./JSONEditorReact.tsx";
import {Content} from "vanilla-jsoneditor";

function App() {
    const [jsonContent, setJsonContent] = useState<Content>({ json: {} })
    const handler = useCallback(
        (content: Content) => {
            setJsonContent(content)
        },
        [jsonContent]
    )

  return (
    <div className="container">
        <JSONEditorReact content={jsonContent} onChange={handler} />
    </div>
  );
}

export default App;
