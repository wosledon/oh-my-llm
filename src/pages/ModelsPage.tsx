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
  Card,
  shorthands,
  tokens,
  Badge,
} from '@fluentui/react-components';
import { Add24Regular, Delete24Regular } from '@fluentui/react-icons';
import { useModelStore } from '../stores/modelStore';
import { useProviderStore } from '../stores/providerStore';
import { useI18n } from '../i18n';
import type { ModelInput } from '../types';

const useStyles = makeStyles({
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  },
  card: {
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    ...shorthands.borderRadius(tokens.borderRadiusXLarge),
    overflow: 'hidden',
  },
  tableRow: {
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  formField: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    marginBottom: '16px',
  },
  empty: {
    textAlign: 'center',
    padding: '48px',
    color: tokens.colorNeutralForeground3,
  },
});

export default function ModelsPage() {
  const styles = useStyles();
  const { t } = useI18n();
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
        <Title1>{t.models.title}</Title1>
        <Dialog open={open} onOpenChange={(_, data: { open: boolean }) => setOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button icon={<Add24Regular />} appearance="primary">{t.models.addModel}</Button>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>{t.models.addModel}</DialogTitle>
              <DialogContent>
                <div className={styles.formField}>
                  <Label>{t.models.provider}</Label>
                  <Dropdown value={form.provider_id} onOptionSelect={(_: unknown, data: { optionValue?: string }) => setForm({ ...form, provider_id: data.optionValue || '' })}>
                    {providers.map((p) => (
                      <Option key={p.id} value={p.id}>{p.name}</Option>
                    ))}
                  </Dropdown>
                </div>
                <div className={styles.formField}>
                  <Label>{t.models.exposedName}</Label>
                  <Input value={form.exposed_name} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, exposed_name: e.target.value })} placeholder={t.models.placeholderExposed} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.models.upstreamName}</Label>
                  <Input value={form.upstream_name} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, upstream_name: e.target.value })} placeholder={t.models.placeholderUpstream} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.models.inputPrice}</Label>
                  <Input type="number" value={String(form.input_price)} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, input_price: parseFloat(e.target.value) || 0 })} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.models.outputPrice}</Label>
                  <Input type="number" value={String(form.output_price)} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, output_price: parseFloat(e.target.value) || 0 })} />
                </div>
                <div className={styles.formField}>
                  <Switch label={t.models.enabled} checked={form.enabled} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, enabled: e.target.checked })} />
                </div>
              </DialogContent>
              <DialogActions>
                <Button appearance="primary" onClick={handleSubmit}>{t.models.save}</Button>
                <Button onClick={() => setOpen(false)}>{t.models.cancel}</Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading && <Spinner label={t.common.loading} />}
      {error && (
        <Badge appearance="filled" color="danger" style={{ marginBottom: '12px', display: 'block' }}>
          {t.common.error}: {error}
        </Badge>
      )}

      <Card className={styles.card}>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHeaderCell>{t.models.exposedName}</TableHeaderCell>
              <TableHeaderCell>{t.models.upstreamName}</TableHeaderCell>
              <TableHeaderCell>{t.models.provider}</TableHeaderCell>
              <TableHeaderCell>{t.models.enabled}</TableHeaderCell>
              <TableHeaderCell>{t.models.inputPrice}</TableHeaderCell>
              <TableHeaderCell>{t.models.outputPrice}</TableHeaderCell>
              <TableHeaderCell>{t.models.actions}</TableHeaderCell>
            </TableRow>
          </TableHeader>
          <TableBody>
            {models.map((m) => (
              <TableRow key={m.id} className={styles.tableRow}>
                <TableCell><b>{m.exposed_name}</b></TableCell>
                <TableCell style={{ fontFamily: 'monospace', fontSize: '12px' }}>{m.upstream_name}</TableCell>
                <TableCell>{providers.find((p) => p.id === m.provider_id)?.name || m.provider_id}</TableCell>
                <TableCell>
                  <Badge appearance="outline" color={m.enabled ? 'success' : 'subtle'}>
                    {m.enabled ? 'Yes' : 'No'}
                  </Badge>
                </TableCell>
                <TableCell>{m.input_price}</TableCell>
                <TableCell>{m.output_price}</TableCell>
                <TableCell>
                  <Button icon={<Delete24Regular />} appearance="subtle" onClick={() => deleteModel(m.id)} />
                </TableCell>
              </TableRow>
            ))}
            {models.length === 0 && (
              <TableRow>
                <TableCell colSpan={7}>
                  <div className={styles.empty}>{t.dashboard.noData}</div>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </Card>
    </div>
  );
}
