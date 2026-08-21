import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { AlertError } from 'app/shared/alert/alert-error';
import { TranslateDirective } from 'app/shared/language';
import { IRadarState } from '../radar-state.model';
import { RadarStateService } from '../service/radar-state.service';

import { RadarStateFormGroup, RadarStateFormService } from './radar-state-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-radar-state-update',
  templateUrl: './radar-state-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class RadarStateUpdate implements OnInit {
  readonly isSaving = signal(false);
  radarState: IRadarState | null = null;

  protected radarStateService = inject(RadarStateService);
  protected radarStateFormService = inject(RadarStateFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: RadarStateFormGroup = this.radarStateFormService.createRadarStateFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ radarState }) => {
      this.radarState = radarState;
      if (radarState) {
        this.updateForm(radarState);
      }
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const radarState = this.radarStateFormService.getRadarState(this.editForm);
    if (radarState.id === null) {
      this.subscribeToSaveResponse(this.radarStateService.create(radarState));
    } else {
      this.subscribeToSaveResponse(this.radarStateService.update(radarState));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IRadarState | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(radarState: IRadarState): void {
    this.radarState = radarState;
    this.radarStateFormService.resetForm(this.editForm, radarState);
  }
}
