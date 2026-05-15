import { useEffect, useState } from 'react';
import {
  Title1,
  Button,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Label,
  Input,
  Dropdown,
  Option,
  Switch,
  makeStyles,
  Spinner,
} from '@fluentui/react-components';
import { Add24Regular, Delete24Regular } from '@fluentui/react-icons';
import { useModelStore } from '../stores/modelStore';
import { useProviderStore } from '../stores/providerStore';
import type { ModelInput } from '../types';

const useStyles = makeStyles({
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  formField: {
    marginBottom: '12px',
  },
});

export default function ModelsPage() {
  const styles = useStyles();
  const { models, loading, error, fetchModels, addModel, deleteModel } = useModelStore();
  const { providers, fetchProviders } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState<ModelInput>({
    provider_id: '',
    exposed_name: '',
    upstream_name: '',
    enabled: true,
    input_price: 0,
    output_price: 0,
  });

  useEffect(() => {
    fetchModels();
    fetchProviders();
  }, [fetchModels, fetchProviders]);

  const handleSubmit = async () => {
    await addModel(form);
    setOpen(false);
    setForm({ provider_id: '', exposed_name: '', upstream_name: '', enabled: true, input_price: 0, output_price: 0 });
  };

  return (
    <div>
      <div className={styles.toolbar}>
        <Title1>Models</Title1>
        <Dialog open={open} onOpenChange={(_, data) => setOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button icon={<Add24Regular />}>Add Model</Button>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>Add Model Mapping</DialogTitle>
              <DialogContent>
                <div className={styles.formField}>
                  <Label>Provider</Label>
                  <Dropdown value={form.provider_id} onOptionSelect={(_, data) => setForm({ ...form, provider_id: data.optionValue || '' })}>
                    {providers.map((p) => (
                      <Option key={p.id} value={p.id}>{p.name}</Option>
                    ))}
                  </Dropdown>
                </div>
                <div className={styles.formField}>
                  <Label>Exposed Name</Label>
                  <Input value={form.exposed_name} onChange={(e) => setForm({ ...form, exposed_name: e.target.value })} placeholder="gpt-4" />
                </div>
                <div className={styles.formField}>
                  <Label>Upstream Name</Label>
                  <Input value={form.upstream_name} onChange={(e) => setForm({ ...form, upstream_name: e.target.value })} placeholder="gpt-4-0613" />
                </div>
                <div className={styles.formField}>
                  <Label>Input Price (USD / 1M tokens)</Label>
                  <Input type="number" value={String(form.input_price)} onChange={(e) => setForm({ ...form, input_price: parseFloat(e.target.value) || 0 })} />
                </div>
                <div className={styles.formField}>
                  <Label>Output Price (USD / 1M tokens)</Label>
                  <Input type="number" value={String(form.output_price)} onChange={(e) => setForm({ ...form, output_price: parseFloat(e.target.value) || 0 })} />
                </div>
                <div className={styles.formField}>
                  <Switch label="Enabled" checked={form.enabled} onChange={(e) => setForm({ ...form, enabled: e.target.checked })} />
                </div>
              </DialogContent>
              <DialogActions>
                <Button appearance="primary" onClick={handleSubmit}>Save</Button>
                <Button onClick={() => setOpen(false)}>Cancel</Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading && <Spinner />}
      {error && <div style={{ color: 'red' }}>{error}</div>}

      <Table>
        <TableHeader>
          <TableRow>
            <TableHeaderCell>Exposed Name</TableHeaderCell>
            <TableHeaderCell>Upstream Name</TableHeaderCell>
            <TableHeaderCell>Provider</TableHeaderCell>
            <TableHeaderCell>Enabled</TableHeaderCell>
            <TableHeaderCell>Input Price</TableHeaderCell>
            <TableHeaderCell>Output Price</TableHeaderCell>
            <TableHeaderCell>Actions</TableHeaderCell>
          </TableRow>
        </TableHeader>
        <TableBody>
          {models.map((m) => (
            <TableRow key={m.id}>
              <TableCell>{m.exposed_name}</TableCell>
              <TableCell>{m.upstream_name}</TableCell>
              <TableCell>{providers.find(p => p.id === m.provider_id)?.name || m.provider_id}</TableCell>
              <TableCell>{m.enabled ? 'Yes' : 'No'}</TableCell>
              <TableCell>{m.input_price}</TableCell>
              <TableCell>{m.output_price}</TableCell>
              <TableCell>
                <Button icon={<Delete24Regular />} onClick={() => deleteModel(m.id)} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
